use std::collections::HashMap;

use alloy::{
    primitives::{Address, U256, aliases::I24, keccak256},
    transports::http::reqwest::Url,
};
use alloy_provider::ProviderBuilder;
use alloy_sol_types::SolValue;

use crate::{
    err::{MathError, TradeError},
    sol_types::{PoolId, PoolKey, StateView::StateViewInstance, V3Pool::V3PoolInstance},
    v3_base::{
        bitmap::BitMap,
        bitmap_math,
        states::{PoolState, Tick, TradeState},
        tick_math,
        ticks::Ticks,
        trade_math::{self, retry},
    },
    v3_pool::{
        v3_key::V3Key,
        v3_live::{V3Contract, fetch_v3_word_ticks, get_v3_ticks},
    },
    v4_pool::{
        v4_key::V4Key,
        v4_live::{V4Contract, fetch_v4_word_ticks, get_v4_ticks},
    },
};

pub enum AnyPool {
    V3(PoolState, V3Key, V3Contract),
    V4(PoolState, V4Key, V4Contract),
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
    pub async fn get_next_word_ticks(trade_state: &TradeState) {}

    pub async fn handle_trade_error(
        &mut self,
        trade_error: TradeError,
        tick_spacing: I24,
    ) -> Result<TradeState, TradeError> {
        let mut ticks_to_fetch: Vec<I24> = Vec::new();
        let mut trade_state = Option::None;
        match trade_error {
            TradeError::Tick(tick_error) => match tick_error {
                crate::err::TickError::Overflow(trade_state) => {
                    let normalized_tick = bitmap_math::normalize_tick(
                        trade_state.step.next_tick.tick,
                        tick_spacing,
                    );
                    let word_idx = I24::try_from(bitmap_math::get_pos_from_tick(
                        trade_state.step.next_tick.tick,
                        tick_spacing,
                    ))
                    .expect("error on handling overflow trading error");

                    let word_bitmap = self
                        .get_word(bitmap_math::word_index(normalized_tick))
                        .await;
                    match word_bitmap {
                        Ok(bitmap) => match self {
                            AnyPool::V3(pool_state, _, _) => {
                                let ticks = bitmap_math::extract_ticks_from_bitmap(
                                    bitmap,
                                    word_idx,
                                    tick_spacing,
                                );
                                pool_state.bitmap.insert(
                                    bitmap_math::get_pos_from_tick(
                                        trade_state.tick + I24::ONE,
                                        tick_spacing,
                                    ),
                                    bitmap,
                                );

                                let new_ticks = self.fetch_ticks(ticks).await.0;
                                self.insert_ticks(new_ticks);
                                /////////////////////////////////////////////////////////
                            }
                            AnyPool::V4(pool_state, _, _) => todo!(),
                        },
                        Err(err) => todo!(),
                    }
                }
                crate::err::TickError::Underflow(trade_state) => todo!(),
                crate::err::TickError::Unavailable(trade_state) => todo!(),
            },
            TradeError::Math(math_error) => {
                return Err(math_error.into());
            }
        }
        return trade_math::retry(trade_state.unwrap(), self.get_ticks());
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
        let normalized_tick =
            bitmap_math::normalize_tick(slot0.tick, pool_key.tickSpacing);
        let word_index = bitmap_math::word_index(normalized_tick);
        let bitmap = contract
            .getTickBitmap(pool_id, word_index)
            .call()
            .await?;

        let active_ticks = fetch_v4_word_ticks(
            &contract,
            bitmap,
            PoolId::from_underlying(pool_id),
            word_index,
            pool_key.tickSpacing,
        )
        .await;
        println!("v4 loaded: {}", slot0.sqrtPriceX96);

        let state = PoolState {
            current_tick: slot0.tick,
            ticks: Ticks::new(active_ticks),
            bitmap: BitMap::new(pool_key.tickSpacing, vec![(word_index, bitmap)]),
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
        let normalized_tick = bitmap_math::normalize_tick(slot0.tick, tick_spacing);

        let liquidity = contract.liquidity().call().await?;
        let word_index = bitmap_math::word_index(normalized_tick);
        let bitmap = contract.tickBitmap(word_index).call().await?;
        let active_ticks = fetch_v3_word_ticks(
            contract.clone(),
            bitmap,
            word_index,
            tick_spacing,
            5_u8,
            6_u64,
        )
        .await;

        println!("v3 price: {}", slot0.sqrtPriceX96);
        let config = V3Key {
            address: addr,
            token0: token_0,
            token1: token_1,
            fee,
            tick_spacing,
        };
        let state = PoolState {
            current_tick: slot0.tick,
            ticks: Ticks::new(active_ticks),
            bitmap: BitMap::new(tick_spacing, vec![(word_index, bitmap)]),

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
    pub async fn get_word(&self, word_pos: i16) -> Result<U256, alloy_contract::Error> {
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
        result
    }
    pub fn get_ticks(&self) -> &Ticks {
        match self {
            AnyPool::V3(pool_state, _, _) => return &pool_state.ticks,
            AnyPool::V4(pool_state, _, _) => return &pool_state.ticks,
        }
    }
    pub async fn fetch_ticks(&self, ticks: Vec<I24>) -> (Vec<Tick>, Vec<usize>) {
        match self {
            AnyPool::V3(_, _, contract) => {
                let (n, f) = get_v3_ticks(contract.clone(), ticks).await;
                return (n, f);
            }
            AnyPool::V4(_, key, contract) => {
                let key = keccak256(key.abi_encode());
                let (n, f) = get_v4_ticks(contract.clone(), ticks, key).await;
                return (n, f);
            }
        }
    }
}
