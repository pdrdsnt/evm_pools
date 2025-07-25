use alloy::primitives::{U256, U512, aliases::U24};

use crate::{
    err::{MathError, TickError, TradeError},
    v3_base::{
        states::{PoolState, TradeState},
        tick_math::{price_from_tick, tick_from_price},
        x96price_math::{
            compute_amount_possible, compute_price_from0,
            compute_price_from1,
        },
    },
};

pub fn trade(
    pool: &PoolState,
    fee: &U24,
    amount_in: U256,
    from0: bool,
) -> Result<TradeState, TradeError> {
    // 1. Fee deduction
    println!(
        "rEmaining before fee {}",
        amount_in
    );

    let mut trade_state = TradeState {
        fee_amount: U256::ZERO,
        remaining: amount_in.clone(),
        x96price: pool.x96price,
        liquidity: pool.liquidity,
        amount_in: amount_in,
        tick: pool.current_tick,
    };
    let fee_amount = amount_in
        .checked_mul(U256::from(*fee))
        .ok_or(MathError::A(trade_state))?
        .checked_div(U256::from(1_000_000))
        .ok_or(MathError::A(trade_state))?;
    let mut remaining = amount_in
        .checked_sub(fee_amount)
        .ok_or(MathError::A(trade_state))?;
    println!(
        "remaining after fee {}",
        remaining
    );

    trade_state.fee_amount = fee_amount;

    // 2. Local state
    let mut total_out = U256::ZERO;

    let mut curr_price = pool.x96price;

    let current_tick = tick_from_price(pool.x96price)
        .ok_or(MathError::A(trade_state))?;

    let mut next_tick_index = match pool
        .active_ticks
        .binary_search_by_key(
            &current_tick,
            |t| t.tick,
        ) {
        Ok(i) => {
            if from0 {
                if i + 1
                    >= pool
                        .active_ticks
                        .len()
                {
                    return Err(
                        TickError::Overflow(trade_state)
                            .into(),
                    );
                } // No ticks above
                i + 1
            } else {
                if i == 0 {
                    return Err(
                        TickError::Underflow(trade_state)
                            .into(),
                    );
                } // No ticks below
                i - 1
            }
        }
        Err(i) => {
            if from0 {
                if i >= pool
                    .active_ticks
                    .len()
                {
                    return Err(
                        TickError::Overflow(trade_state)
                            .into(),
                    );
                } // No ticks above
                i
            } else {
                if i == 0 {
                    return Err(
                        TickError::Underflow(trade_state)
                            .into(),
                    );
                } // No ticks below
                i - 1
            }
        }
    };
    println!(
        "next tick: {}",
        next_tick_index
    );
    let mut curr_liq = pool.liquidity;

    while remaining > U256::ZERO {
        // just entering a new step
        println!(
            "── loop step, next_tick_index = {}",
            next_tick_index
        );

        // grab the next tick struct
        let next_tick = pool
            .active_ticks
            .get(next_tick_index as usize)
            .ok_or(MathError::A(trade_state))?;
        println!(
            "next_tick: tick={} liquidity_net={:?}",
            next_tick.tick, next_tick.liquidity_net
        );

        if next_tick
            .liquidity_net
            .is_none()
        {
            return Err(
                TickError::Unavailable(trade_state).into(),
            );
        }
        // calculate the next tick’s price
        let next_price = price_from_tick(next_tick.tick)
            .ok_or(MathError::A(trade_state))?;
        println!(
            "calculated next_price from tick {}: {}",
            next_tick.tick, next_price
        );

        // calculate the current price (based on current_tick)
        let current_price_v = price_from_tick(current_tick)
            .ok_or(MathError::A(trade_state))?;
        println!(
            "calculated current_price from current_tick {}: {}",
            current_tick, current_price_v
        );
        let old_tick_index = next_tick_index;
        next_tick_index = if from0 {
            next_tick_index
                .checked_add(1)
                .ok_or(TickError::Overflow(trade_state))?
        } else {
            next_tick_index
                .checked_sub(1)
                .ok_or(TickError::Underflow(trade_state))?
        };
        println!(
            "IN RANGE LIQUIDITY {}",
            curr_liq,
        );
        // compute max amount possible to cross this tick
        let possible = compute_amount_possible(
            from0,
            &curr_liq,
            &curr_price,
            &next_price,
        )
        .ok_or(MathError::A(trade_state))?;

        // **DEBUG PRINT**
        println!(
            "[DEBUG] tick_index={} → next_index={} | curr_price={} | next_price={} | \
         curr_liq={} | remaining={} | possible_to_cross={}",
            old_tick_index,
            next_tick_index,
            curr_price,
            next_price,
            curr_liq,
            remaining,
            possible
        );

        if remaining < possible {
            // won't cross full tick
            let new_price = if from0 {
                compute_price_from0(
                    &remaining,
                    &curr_liq,
                    &curr_price,
                    true,
                )
                .ok_or(MathError::A(trade_state))?
            } else {
                compute_price_from1(
                    &remaining,
                    &curr_liq,
                    &curr_price,
                    true,
                )
                .ok_or(MathError::A(trade_state))?
            };

            // **DEBUG PRINT**
            println!(
                "[DEBUG] crossing partial tick: remaining={} < possible={}, new_price={}",
                remaining, possible, new_price
            );

            let u512_curr_price = U512::from(curr_price);
            let u512_curr_liq = U512::from(curr_liq);

            // compute out
            let delta = if from0 {
                let price_diff = u512_curr_price
                    .checked_sub(U512::from(new_price))
                    .ok_or(MathError::A(trade_state))?;
                println!(
                    "diff {}",
                    price_diff
                );
                u512_curr_liq
                    .checked_mul(price_diff)
                    .ok_or(MathError::A(trade_state))?
                    .checked_div(U512::ONE << 96)
                    .ok_or(MathError::A(trade_state))?
            } else {
                let inv_curr = (U512::ONE
                    << U512::from(96_u32))
                .checked_mul(U512::ONE << 96)
                .ok_or(MathError::A(trade_state))?
                .checked_div(u512_curr_price)
                .ok_or(MathError::A(trade_state))?;
                let inv_new = (U512::ONE
                    << U512::from(96_u32))
                .checked_mul(U512::ONE << 96)
                .ok_or(MathError::A(trade_state))?
                .checked_div(U512::from(new_price))
                .ok_or(MathError::A(trade_state))?;
                u512_curr_liq
                    .checked_mul(
                        inv_curr
                            .checked_sub(inv_new)
                            .ok_or(
                                MathError::A(trade_state),
                            )?,
                    )
                    .ok_or(MathError::A(trade_state))?
                    .checked_div(U512::from(1u128 << 96))
                    .ok_or(MathError::A(trade_state))?
            };

            println!(
                "delta {}",
                delta
            );

            total_out = total_out
                .checked_add(U256::from(delta))
                .ok_or(MathError::A(trade_state))?;
            remaining = U256::ZERO;
            curr_price = U256::from(new_price);

            // **DEBUG PRINT**
            println!(
                "[DEBUG] partial-cross: delta_out={} | total_out={} | curr_price(updated)={} | remaining=0",
                delta, total_out, curr_price
            );

            trade_state.x96price = curr_price;
            trade_state.liquidity = curr_liq;
            trade_state.remaining = remaining;

            break;
        }

        // cross entire tick
        let out_cross = if from0 {
            curr_liq
                .checked_mul(
                    next_price
                        .checked_sub(curr_price)
                        .ok_or(MathError::A(trade_state))?,
                )
                .ok_or(MathError::A(trade_state))?
                .checked_div(U256::from(1u128 << 96))
                .ok_or(MathError::A(trade_state))?
        } else {
            let num = curr_liq
                .checked_mul(
                    curr_price
                        .checked_sub(next_price)
                        .ok_or(MathError::A(trade_state))?,
                )
                .ok_or(MathError::A(trade_state))?;
            num.checked_div(U256::from(1u128 << 96))
                .ok_or(MathError::A(trade_state))?
        };
        total_out = total_out
            .checked_add(out_cross)
            .ok_or(MathError::A(trade_state))?;

        // **DEBUG PRINT**
        println!(
            "[DEBUG] full-cross: out_cross={} | total_out(before_liq_update)={} ",
            out_cross,
            total_out - out_cross
        );

        // update liquidity
        if let Some(net) = next_tick.liquidity_net {
            let old_liq = curr_liq;
            curr_liq = if from0 {
                if net > 0 {
                    curr_liq.saturating_add(U256::from(net))
                } else {
                    curr_liq
                        .saturating_sub(U256::from(-net))
                }
            } else {
                if net < 0 {
                    curr_liq.saturating_add(U256::from(net))
                } else {
                    curr_liq.saturating_sub(U256::from(net))
                }
            };
            // **DEBUG PRINT**
            println!(
                "[DEBUG] liquidity_net={} | liquidity: {} → {}",
                net, old_liq, curr_liq
            );
        }

        // move pointer
        curr_price = next_price;
        remaining = remaining
            .checked_sub(possible)
            .ok_or(MathError::A(trade_state))?;

        // **DEBUG PRINT**
        println!(
            "[DEBUG] end-of-iteration: curr_price={} | remaining={} | total_out={}\n",
            curr_price, remaining, total_out
        );

        trade_state.x96price = curr_price;
        trade_state.liquidity = curr_liq;
        trade_state.remaining = remaining;
    }

    // build Trade
    Ok(trade_state)
}
