use alloy::primitives::{
    U256, U512,
    aliases::{I24, U24},
};

use crate::{
    err::{MathError, TickError, TradeError},
    v3_base::{
        states::{PoolState, Tick, TradeState, TradeStep},
        tick_math::{price_from_tick, tick_from_price},
        ticks::Ticks,
        x96price_math::{
            compute_amount_possible, compute_price_from0, compute_price_from1,
        },
    },
};

pub fn trade(
    pool: &PoolState,
    fee: &U24,
    amount_in: U256,
    from0: bool,
) -> Result<TradeState, TradeError> {
    let mut trade_state = trade_start(pool, fee, amount_in, from0)?;
    while trade_state.remaining > U256::ZERO {
        trade_state = step_start(trade_state, &pool.ticks)?;
        if trade_state.remaining < trade_state.step.amount_possible {
            match handle_non_crossing_step(trade_state) {
                Ok(ts) => trade_state = ts,
                Err(err) => return Err(err),
            }
            break;
        }

        // cross entire tick
        let out_cross = if from0 {
            trade_state
                .liquidity
                .checked_mul(
                    trade_state
                        .step
                        .next_price
                        .checked_sub(trade_state.x96price)
                        .ok_or(MathError::A(trade_state))?,
                )
                .ok_or(MathError::A(trade_state))?
                .checked_div(U256::from(1u128 << 96))
                .ok_or(MathError::A(trade_state))?
        } else {
            let num = trade_state
                .liquidity
                .checked_mul(
                    trade_state
                        .x96price
                        .checked_sub(trade_state.step.next_price)
                        .ok_or(MathError::A(trade_state))?,
                )
                .ok_or(MathError::A(trade_state))?;
            num.checked_div(U256::from(1u128 << 96))
                .ok_or(MathError::A(trade_state))?
        };
        trade_state.amount_out = trade_state
            .amount_out
            .checked_add(out_cross)
            .ok_or(MathError::A(trade_state))?;
        // update liquidity
        if let Some(net) = trade_state.step.next_tick.liquidity_net {
            trade_state.liquidity = if from0 {
                if net > 0 {
                    trade_state
                        .liquidity
                        .saturating_add(U256::from(net))
                } else {
                    trade_state
                        .liquidity
                        .saturating_sub(U256::from(-net))
                }
            } else {
                if net < 0 {
                    trade_state
                        .liquidity
                        .saturating_add(U256::from(net))
                } else {
                    trade_state
                        .liquidity
                        .saturating_sub(U256::from(net))
                }
            };
        }

        // move pointer
        trade_state.x96price = trade_state.step.next_price;
        trade_state.remaining = trade_state
            .remaining
            .checked_sub(trade_state.step.amount_possible)
            .ok_or(MathError::A(trade_state))?;
    }

    // build Trade
    Ok(trade_state)
}

//pub fn trade_step(trade_state: TradeState) -> Result<TradeState, TradeError> {}
pub fn handle_non_crossing_step(
    mut trade_state: TradeState,
) -> Result<TradeState, TradeError> {
    // won't cross full tick
    let new_price = if trade_state.from0 {
        compute_price_from0(
            &trade_state.remaining,
            &trade_state.liquidity,
            &trade_state.x96price,
            true,
        )
        .ok_or(MathError::A(trade_state.clone()))?
    } else {
        compute_price_from1(
            &trade_state.remaining,
            &trade_state.liquidity,
            &trade_state.x96price,
            true,
        )
        .ok_or(MathError::A(trade_state))?
    };

    let u512_curr_price = U512::from(trade_state.x96price);
    let u512_curr_liq = U512::from(trade_state.liquidity);

    // compute out
    let delta = if trade_state.from0 {
        let price_diff = U512::from(new_price)
            .checked_sub(u512_curr_price)
            .ok_or(MathError::A(trade_state))?;
        println!("diff {}", price_diff);
        u512_curr_liq
            .checked_mul(price_diff)
            .ok_or(MathError::A(trade_state))?
            .checked_div(U512::ONE << 96)
            .ok_or(MathError::A(trade_state))?
    } else {
        let inv_curr = (U512::ONE << U512::from(96_u32))
            .checked_mul(U512::ONE << 96)
            .ok_or(MathError::A(trade_state))?
            .checked_div(u512_curr_price)
            .ok_or(MathError::A(trade_state))?;
        let inv_new = (U512::ONE << U512::from(96_u32))
            .checked_mul(U512::ONE << 96)
            .ok_or(MathError::A(trade_state))?
            .checked_div(U512::from(new_price))
            .ok_or(MathError::A(trade_state))?;
        u512_curr_liq
            .checked_mul(
                inv_curr
                    .checked_sub(inv_new)
                    .ok_or(MathError::A(trade_state))?,
            )
            .ok_or(MathError::A(trade_state))?
            .checked_div(U512::from(1u128 << 96))
            .ok_or(MathError::A(trade_state))?
    };

    trade_state.amount_out = trade_state
        .amount_out
        .checked_add(U256::from(delta))
        .ok_or(MathError::A(trade_state))?;
    trade_state.remaining = U256::ZERO;
    trade_state.x96price = U256::from(new_price);

    Ok(trade_state)
}
pub fn step_start(
    mut trade_state: TradeState,
    ticks: &Ticks,
) -> Result<TradeState, TradeError> {
    let mut new_step = TradeStep::default();
    trade_state.step.next_tick_index = match ticks.get_tick_index(trade_state.tick) {
        Ok(i) => {
            if trade_state.from0 {
                if i + 1 >= ticks.len() {
                    return Err(TickError::Overflow(trade_state).into());
                } // No ticks above
                i + 1
            } else {
                if i == 0 {
                    return Err(TickError::Underflow(trade_state).into());
                } // No ticks below
                i - 1
            }
        }
        Err(i) => {
            if trade_state.from0 {
                if i >= ticks.len() {
                    return Err(TickError::Overflow(trade_state).into());
                } // No ticks above
                i
            } else {
                if i == 0 {
                    return Err(TickError::Underflow(trade_state).into());
                } // No ticks below
                i - 1
            }
        }
    };

    trade_state.step.next_tick = *ticks
        .get(trade_state.step.next_tick_index)
        .expect("checked above");

    if trade_state.step.next_tick.liquidity_net.is_none() {
        return Err(TickError::Unavailable(trade_state).into());
    }
    // calculate the next tick’s price
    trade_state.step.next_price = price_from_tick(trade_state.step.next_tick.tick)
        .ok_or(MathError::A(trade_state))?;

    // compute max amount possible to cross this tick
    trade_state.step.amount_possible = compute_amount_possible(
        trade_state.from0,
        &trade_state.liquidity,
        &trade_state.x96price,
        &trade_state.step.next_price,
    )
    .ok_or(MathError::A(trade_state))?;

    Ok(trade_state)
}

pub fn trade_start(
    pool: &PoolState,
    fee: &U24,
    amount_in: U256,
    from0: bool,
) -> Result<TradeState, TradeError> {
    let mut trade_state = TradeState {
        fee_amount: U256::ZERO,
        remaining: amount_in.clone(),
        amount_out: U256::ZERO,
        x96price: pool.x96price,
        liquidity: pool.liquidity,
        amount_in: amount_in,
        tick: pool.current_tick,
        from0: from0,
        step: TradeStep::default(),
    };
    let fee_amount = amount_in
        .checked_mul(U256::from(*fee))
        .ok_or(MathError::A(trade_state))?
        .checked_div(U256::from(1_000_000))
        .ok_or(MathError::A(trade_state))?;
    trade_state.remaining = amount_in
        .checked_sub(fee_amount)
        .ok_or(MathError::A(trade_state))?;

    trade_state.fee_amount = fee_amount;

    trade_state.x96price = pool.x96price;

    trade_state.tick = tick_from_price(pool.x96price).ok_or(MathError::A(trade_state))?;

    trade_state.liquidity = pool.liquidity;
    Ok(trade_state)
}
