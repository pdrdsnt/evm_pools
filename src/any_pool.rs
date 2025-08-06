use std::time::Duration;

use alloy::{
    primitives::{Address, U256, aliases::I24, keccak256},
    transports::http::reqwest::Url,
};
use alloy_provider::ProviderBuilder;
use alloy_sol_types::SolValue;
use tokio::time::sleep;

use crate::{
    err::TradeError,
    sol_types::{PoolKey, StateView::StateViewInstance, V3Pool::V3PoolInstance},
    v3_base::{
        bitmap::BitMap,
        bitmap_math,
        states::{Tick, TradeState, V3State},
        ticks::Ticks,
        trade_math::{self},
    },
    v3_pool::{
        v3_key::V3Key,
        v3_live::{V3Contract, get_v3_ticks},
    },
    v4_pool::{
        v4_key::V4Key,
        v4_live::{V4Contract, get_v4_ticks},
    },
};

pub enum AnyPool {
    V3(V3State, V3Key, V3Contract),
    V4(V3State, V4Key, V4Contract),
}

impl AnyPool {
    pub async fn trade(
        &mut self,
        amount_in: U256,
        from0: bool,
    ) -> Result<TradeState, TradeError> {
        //needed to fetch new words
        let tick_spacing = match self {
            AnyPool::V3(_, k, _) => k.tick_spacing,
            AnyPool::V4(_, k, _) => k.tickSpacing,
        };
        let result = match self {
            AnyPool::V3(pool_state, v3_key, _) => {
                trade_math::trade(&pool_state, &v3_key.fee, amount_in, from0)
            }
            AnyPool::V4(pool_state, pool_key, _) => {
                trade_math::trade(&pool_state, &pool_key.fee, amount_in, from0)
            }
        };

        let trade_state = match result {
            Ok(ts) => ts,
            Err(err) => {
                return self.handle_trade_error(err, tick_spacing).await;
            }
        };

        Ok(trade_state)
    }

    pub async fn handle_trade_error(
        &mut self,
        trade_error: TradeError,
        tick_spacing: I24,
    ) -> Result<TradeState, TradeError> {
        let state;
        match trade_error {
            TradeError::Tick(tick_error) => match tick_error {
                crate::err::TickError::Overflow(trade_state) => {
                    println!("overflow");
                    let new_word_pos =
                        bitmap_math::get_pos_from_tick(trade_state.tick, tick_spacing)
                            + 1;
                    self.fetch_word_and_insert_ticks(new_word_pos)
                        .await?;
                    state = Some(trade_state);
                }
                crate::err::TickError::Underflow(trade_state) => {
                    let new_word_pos =
                        bitmap_math::get_pos_from_tick(trade_state.tick, tick_spacing)
                            - 1;
                    self.fetch_word_and_insert_ticks(new_word_pos)
                        .await?;
                    state = Some(trade_state);
                }
                crate::err::TickError::Unavailable(trade_state) => {
                    self.fetch_and_insert_ticks(
                        vec![trade_state.step.next_tick.tick],
                        2_u8,
                        10_u64,
                    )
                    .await;

                    state = Some(trade_state);
                }
            },
            //
            TradeError::Math(math_error) => {
                return Err(math_error.into());
            }
            TradeError::Fetch(err) => return Err(TradeError::Fetch(err)),
        }
        return trade_math::retry(state.unwrap(), self.get_ticks());
    }

    pub async fn create_v4(
        pool_key: PoolKey,
        provider_url: Url,
        contract_addr: Address,
    ) -> Result<AnyPool, alloy_contract::Error> {
        let provider = ProviderBuilder::new().connect_http(provider_url);
        let contract = StateViewInstance::new(contract_addr, provider);
        let encoded_key = pool_key.abi_encode();
        let pool_id = keccak256(encoded_key);
        let slot0 = contract.getSlot0(pool_id).call().await?;
        let liquidity = contract.getLiquidity(pool_id).call().await?;
        let state = V3State {
            current_tick: slot0.tick,
            ticks: Ticks::new(vec![]),
            liquidity: U256::from(liquidity),
            x96price: U256::from(slot0.sqrtPriceX96),
        };
        let any_pool = AnyPool::V4(state, pool_key, contract);

        Ok(any_pool)
    }
    pub async fn create_v3(
        provider_url: Url,
        addr: Address,
    ) -> Result<AnyPool, alloy_contract::Error> {
        let provider = ProviderBuilder::new().connect_http(provider_url);
        let contract = V3PoolInstance::new(addr, provider);

        let token_0 = contract.token0().call().await?;
        let token_1 = contract.token1().call().await?;
        let slot0 = contract.slot0().call().await?;
        let fee = contract.fee().call().await?;
        let tick_spacing = contract.tickSpacing().call().await?;

        let liquidity = contract.liquidity().call().await?;
        println!("v3 price: {}", slot0.sqrtPriceX96);
        let config = V3Key {
            address: addr,
            token0: token_0,
            token1: token_1,
            fee,
            tick_spacing,
        };
        let state = V3State {
            current_tick: slot0.tick,
            ticks: Ticks::new(vec![]),

            liquidity: U256::from(liquidity),
            x96price: U256::from(slot0.sqrtPriceX96),
        };
        let any_pool = AnyPool::V3(state, config, contract);

        Ok(any_pool)
    }

    pub fn insert_ticks(&mut self, new_ticks: Vec<Tick>) {
        match self {
            AnyPool::V3(pool_state, _, _) => pool_state.ticks.insert_ticks(new_ticks),
            AnyPool::V4(pool_state, _, _) => pool_state.ticks.insert_ticks(new_ticks),
        }
    }

    pub async fn fetch_word_and_insert_ticks(
        &mut self,
        word_pos: i16,
    ) -> Result<(), alloy_contract::Error> {
        let result = match self {
            AnyPool::V3(_, _, v3_pool_instance) => {
                v3_pool_instance.tickBitmap(word_pos).call().await
            }

            AnyPool::V4(_, pool_key, state_view_instance) => {
                state_view_instance
                    .getTickBitmap(keccak256(pool_key.abi_encode()), word_pos)
                    .call()
                    .await
            }
        };
        if let Ok(word) = result {
            let ts;
            match self {
                AnyPool::V3(pool_state, k, _) => ts = k.tick_spacing,
                AnyPool::V4(pool_state, k, _) => ts = k.tickSpacing,
            }
            let ticks = bitmap_math::extract_ticks_from_bitmap(word, word_pos, ts);
            self.fetch_and_insert_ticks(ticks, 3, 10).await;
        }
        Ok(())
    }
    pub fn get_ticks(&self) -> &Ticks {
        match self {
            AnyPool::V3(pool_state, _, _) => return &pool_state.ticks,
            AnyPool::V4(pool_state, _, _) => return &pool_state.ticks,
        }
    }
    pub async fn fetch_and_insert_ticks(
        &mut self,
        ticks: Vec<I24>,
        max_tries: u8,
        duration: u64,
    ) {
        let (mut n, mut f) = self.fetch_ticks(&ticks).await;
        let mut t = 0;
        let mut new_ticks = n.clone();
        let mut rmp = f.clone();

        while t < max_tries && !f.is_empty() {
            let recall: Vec<I24> = {
                let mut _r = vec![];
                for tick_index in f.iter() {
                    let sel = new_ticks[*tick_index].tick;
                    _r.push(sel);
                }

                _r
            };
            sleep(Duration::from_secs(duration)).await;
            println!("try: {}", t);
            t += 1;

            (n, f) = self.fetch_ticks(&recall).await;
            for (i, &orig_idx) in rmp.iter().enumerate() {
                new_ticks[orig_idx].liquidity_net = n[i].liquidity_net;
            }

            let mut _rmp = Vec::with_capacity(rmp.len());
            for fail in f.iter() {
                _rmp.push(rmp[*fail]);
                println!("new: {} - is old: {}", fail, rmp[*fail]);
            }
            rmp = _rmp;
        }

        self.insert_ticks(new_ticks);
    }

    pub async fn fetch_ticks(&mut self, ticks: &Vec<I24>) -> (Vec<Tick>, Vec<usize>) {
        match self {
            AnyPool::V3(_, _, contract) => get_v3_ticks(contract.clone(), ticks).await,
            AnyPool::V4(_, key, contract) => {
                let key = keccak256(key.abi_encode());
                get_v4_ticks(contract.clone(), ticks, key).await
            }
        }
    }
}
