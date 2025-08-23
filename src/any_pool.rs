use std::{
    collections::HashMap,
    fmt::{write, Debug},
};

use alloy::primitives::{
    aliases::{I24, U112, U24},
    keccak256, Address, B256, U160, U256,
};
use alloy_provider::Provider;
use alloy_sol_types::SolValue;
use futures::{future::join_all, stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::join;

use crate::{
    any_trade::AnyTrade,
    pool_contract::PoolContract,
    sol_types::{
        IUniswapV2Pair::IUniswapV2PairInstance, PoolKey, StateView::StateViewInstance,
        V3Pool::V3PoolInstance,
    },
    v2_pool::V2Pool,
    v3_base::{
        bitmap_math, states::V3State, ticks::Tick, trade_math::trade_start, v3_state,
    },
};
pub enum AnyPool<P: Provider + Clone> {
    V2(V2Pool, IUniswapV2PairInstance<P>),
    V3(V3State, V4Key, Address, V3PoolInstance<P>),
    V4(V3State, V4Key, B256, StateViewInstance<P>),
}

impl<P: Provider + Clone> AnyPool<P> {
    pub fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) -> Result<AnyTrade, crate::err::TradeError> {
        let state: &V3State;
        let fee: U24;
        match self {
            AnyPool::V2(v2_pool, _) => match v2_pool.trade(amount, from0) {
                Some(ok) => Ok(AnyTrade::V2(ok)),
                None => Err(crate::err::TradeError::V2),
            },
            AnyPool::V3(v3_state, v4_key, _, _) => {
                state = v3_state;
                fee = v4_key.fee;
                match trade_start(&state, &fee, amount, from0) {
                    Ok(ok) => Ok(AnyTrade::V3(ok)),
                    Err(err) => Err(err),
                }
            }
            AnyPool::V4(v3_state, v4_key, _, _) => {
                state = v3_state;
                fee = v4_key.fee;
                match trade_start(&state, &fee, amount, from0) {
                    Ok(ok) => Ok(AnyTrade::V4(ok)),
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub async fn create_v2_from_address(
        //v2 receives a fee since is hard coded,
        //we need to match know factories with fees before calling
        //or passing the list here and calling factory()
        //or just using the uniswap default 3000
        addr: Address,
        fee: Option<u32>,
        provider: P,
    ) -> Option<AnyPool<P>> {
        let contract = IUniswapV2PairInstance::new(addr, provider);
        let mut state = V2Pool::default();
        let mut one_valid_answer = false;

        if let Ok(r) = contract.getReserves().call().await {
            let r0 = U256::from(r.reserve0);
            let r1 = U256::from(r.reserve1);
            if r0 != U256::ZERO && r1 != U256::ZERO {
                one_valid_answer = true;
                state.reserves0 = r0;
                state.reserves1 = r1;
            }
        }

        if !one_valid_answer {
            return None;
        }

        if let Ok(t0) = contract.token0().call().await {
            state.token0 = t0;
        }
        if let Ok(t1) = contract.token1().call().await {
            state.token1 = t1;
        }

        if let Some(f) = fee {
            state.fee = f;
        } else {
            state.fee = 3000;
        }

        Some(Self::V2(state, contract))
    }

    pub async fn create_v3_from_address(
        //v3 reveives a provider and an address
        addr: Address,
        provider: P,
    ) -> Option<AnyPool<P>> {
        let fee_to_spacing = HashMap::from([
            (100, 1),  // 0.01%
            (500, 10), // 0.05%
            (2500, 50),
            (3000, 60),   // 0.30%
            (10000, 200), // 1.00%
        ]);
        let contract = V3PoolInstance::new(addr, provider);
        let mut state = V3State::default();
        let mut one_valid_answer = false;

        if let Ok(liquidity) = contract.liquidity().call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity != 0 {
                one_valid_answer = true;
            }
        }

        if let Ok(slot0) = contract.slot0().call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                one_valid_answer = true;
            }
        }
        if !one_valid_answer {
            return None;
        }

        let mut key = V4Key::default();

        if let Ok(fee) = contract.fee().call().await {
            key.fee = fee;
        }
        if let Ok(t0) = contract.token0().call().await {
            key.currency0 = t0;
        }
        if let Ok(t1) = contract.token1().call().await {
            key.currency1 = t1;
        }
        if let Ok(ts) = contract.tickSpacing().call().await {
            key.tickSpacing = ts;
        } else {
            if let Some(saved_spacing) = fee_to_spacing.get(&key.fee.to()) {
                if let Ok(spacing) = I24::try_from(*saved_spacing) {
                    key.tickSpacing = spacing;
                }
            }
        }

        Self::update_v3_words(&mut state, key.tickSpacing, contract.clone(), true).await;

        Some(Self::V3(state, key, addr, contract))
    }

    pub async fn create_v4_from_key(
        //v4 receives a contract directly since
        //it is one manager for all pool in one dex
        //so we dont need a pair contract, only a key
        contract: StateViewInstance<P>,
        key: PoolKey,
    ) -> Option<AnyPool<P>> {
        let mut state = V3State::default();
        let mut one_valid_answer = false;

        let id = keccak256(key.abi_encode());
        if let Ok(liquidity) = contract.getLiquidity(id).call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity != 0 {
                one_valid_answer = true;
            }
        }

        if let Ok(slot0) = contract.getSlot0(id).call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                one_valid_answer = true;
            }
        }

        if !one_valid_answer {
            return None;
        }
        Some(Self::V4(state, key.into(), id, contract))
    }
    pub async fn sync_v2(
        state: &mut V2Pool,
        contract: IUniswapV2PairInstance<P>,
    ) -> Result<(), ()> {
        if let Ok(liquidity) = contract.getReserves().call().await {
            state.reserves0 = U256::from(liquidity.reserve0);
            state.reserves1 = U256::from(liquidity.reserve1);

            if liquidity.reserve0 == U112::ZERO {
                return Err(());
            }
        }

        Err(())
    }
    pub async fn sync_v3(
        state: &mut V3State,
        contract: V3PoolInstance<P>,
    ) -> Result<(), ()> {
        if let Ok(liquidity) = contract.liquidity().call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity == 0 {
                return Err(());
            }
        }

        if let Ok(slot0) = contract.slot0().call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                return Err(());
            }
            return Ok(());
        }
        Err(())
    }

    pub async fn sync_v4(
        state: &mut V3State,
        contract: StateViewInstance<P>,
        id: B256,
    ) -> Result<(), ()> {
        if let Ok(liquidity) = contract.getLiquidity(id).call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity == 0 {
                return Err(());
            }
        }

        if let Ok(slot0) = contract.getSlot0(id).call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                return Err(());
            }
            return Ok(());
        }
        Err(())
    }

    pub async fn update_v3_words(
        state: &mut V3State,
        tick_spacing: I24,
        contract: V3PoolInstance<P>,
        update: bool,
    ) -> Result<(), ()> {
        let w_idx = bitmap_math::word_index(bitmap_math::normalize_tick(
            state.tick,
            tick_spacing,
        ));

        let words_around = [
            w_idx - 1,
            w_idx,
            w_idx + 1,
        ];
        let mut active_ticks = Vec::<I24>::new();
        for word_idx in words_around {
            let mut bm = state
                .bitmap
                .get_word_from_tick(state.tick, tick_spacing);

            if bm.is_none() || update {
                if let Ok(bitmap) = contract.tickBitmap(word_idx).call().await {
                    bm = Some(bitmap);
                    state
                        .bitmap
                        .insert(word_idx, bitmap, tick_spacing);
                } else {
                    continue;
                };
            }

            if let Some(bitmap) = bm {
                let mut bitmap_ticks = bitmap_math::extract_ticks_from_bitmap(
                    bitmap,
                    word_idx,
                    tick_spacing,
                );
                active_ticks.append(&mut bitmap_ticks);
            }
        }
        Self::fetch_and_insert_v3_ticks(state, contract, active_ticks, update).await;

        Ok(())
    }
    pub async fn fetch_and_insert_v3_ticks(
        state: &mut V3State,
        contract: V3PoolInstance<P>,
        ticks: Vec<I24>,
        update: bool,
    ) {
        let mut fut = FuturesUnordered::from(
            ticks
                .iter()
                .filter_map(|x| {
                    let call = async { (x.clone(), contract.ticks(*x).call().await) };
                    if update {
                        //force update
                        return Some(call);
                    } else {
                        //try load first
                        if let Ok(tick) = state.ticks.get_tick(*x) {
                            if tick.liquidity_net.is_some() {
                                return None;
                            }
                            return Some(call);
                        }
                        None
                    }
                })
                .collect(),
        );
        let mut tcks = Vec::new();

        while let Some((tikc, tick_response)) = fut.next().await {
            let mut net = None;
            if let Ok(tr) = tick_response {
                net = Some(tr.liquidityNet);
            }

            let tick = Tick {
                tick: tikc,
                liquidity_net: net,
            };
            tcks.push(tick);
        }

        state.ticks.insert_ticks(tcks);
    }
    pub async fn update_v4_words(
        state: &mut V3State,
        tick_spacing: I24,
        id: B256,
        contract: StateViewInstance<P>,
        update: bool,
    ) -> Result<(), ()> {
        let w_idx = bitmap_math::word_index(bitmap_math::normalize_tick(
            state.tick,
            tick_spacing,
        ));
        let words_around = [
            w_idx - 1,
            w_idx,
            w_idx + 1,
        ];
        let mut active_ticks = Vec::<I24>::new();
        for word_idx in words_around {
            let mut bm = state
                .bitmap
                .get_word_from_tick(state.tick, tick_spacing);
            if bm.is_none() || update {
                if let Ok(bitmap) = contract.getTickBitmap(id, word_idx).call().await {
                    bm = Some(bitmap);
                    state
                        .bitmap
                        .insert(word_idx, bitmap, tick_spacing);
                } else {
                    continue;
                };
            }

            if let Some(bitmap) = bm {
                let mut bitmap_ticks = bitmap_math::extract_ticks_from_bitmap(
                    bitmap,
                    word_idx,
                    tick_spacing,
                );
                active_ticks.append(&mut bitmap_ticks);
            }
        }
        Self::fetch_and_insert_v4_ticks(state, contract, id, active_ticks, update).await;

        Ok(())
    }
    pub async fn fetch_and_insert_v4_ticks(
        state: &mut V3State,
        contract: StateViewInstance<P>,
        id: B256,
        ticks: Vec<I24>,
        update: bool,
    ) {
        let mut fut = FuturesUnordered::from(
            ticks
                .iter()
                .filter_map(|x| {
                    let call = {
                        async { (x.clone(), contract.getTickInfo(id, *x).call().await) }
                    };
                    if update {
                        Some(call)
                    } else {
                        if let Ok(tick) = state.ticks.get_tick(*x) {
                            if tick.liquidity_net.is_some() {
                                //do not request for this tick
                                return None;
                            }
                            return Some(call);
                        }
                        None
                    }
                })
                .collect(),
        );
        let mut tcks = Vec::new();

        while let Some((tikc, tick_response)) = fut.next().await {
            let mut net = None;
            if let Ok(tr) = tick_response {
                net = Some(tr.liquidityNet);
            }

            let tick = Tick {
                tick: tikc,
                liquidity_net: net,
            };
            tcks.push(tick);
        }

        state.ticks.insert_ticks(tcks);
    }
}
impl<P: Provider + Clone> Debug for AnyPool<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyPool::V2(v2_pool, _) => {
                write!(
                    f,
                    "V2 {} \n token 0: {} - {}\n token 1: {} - {}",
                    v2_pool.address,
                    v2_pool.token0,
                    v2_pool.reserves0,
                    v2_pool.token1,
                    v2_pool.reserves1
                )
            }
            AnyPool::V3(v3_state, v4_key, address, _) => {
                write!(
                    f,
                    "V3 {} fee - {} \n token 0: {} \n token 1: {}\n price {} \n liquidity {}",
                    address,
                    v4_key.fee,
                    v4_key.currency0,
                    v4_key.currency1,
                    v3_state.x96price,
                    v3_state.liquidity
                )
            }
            AnyPool::V4(v3_state, v4_key, fixed_bytes,_) => write!(
                f,
                "V4 {} fee - {} - tick spacing {}\n token 0: {} \n token 1: {}\n price {} \n liquidity {}",
                fixed_bytes,
                v4_key.fee,
                v4_key.tickSpacing,
                v4_key.currency0,
                v4_key.currency1,
                v3_state.x96price,
                v3_state.liquidity
            ),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct V4Key {
    currency0: Address,
    currency1: Address,
    fee: U24,
    tickSpacing: I24,
    hooks: Address,
}

impl Into<PoolKey> for V4Key {
    fn into(self) -> PoolKey {
        PoolKey {
            currency0: self.currency0,
            currency1: self.currency1,
            fee: self.fee,
            tickSpacing: self.tickSpacing,
            hooks: self.hooks,
        }
    }
}

impl From<PoolKey> for V4Key {
    fn from(value: PoolKey) -> Self {
        Self {
            currency0: value.currency0,
            currency1: value.currency1,
            fee: value.fee,
            tickSpacing: value.tickSpacing,
            hooks: value.hooks,
        }
    }
}
