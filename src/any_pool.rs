use std::collections::HashMap;

use alloy::{
    primitives::{Address, U256, aliases::I24, keccak256},
    transports::http::reqwest::Url,
};
use alloy_provider::ProviderBuilder;
use alloy_sol_types::SolValue;
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{
    sol_types::{
        PoolId, PoolKey, StateView::StateViewInstance,
        V3Pool::V3PoolInstance,
    },
    v3_base::{
        bitmap_math,
        states::{PoolState, Tick},
    },
    v3_pool::{
        v3_key::V3Key,
        v3_live::{
            V3Contract, fetch_v3_word_ticks, get_v3_ticks,
        },
    },
    v4_pool::{
        v4_key::V4Key,
        v4_live::{
            V4Contract, fetch_v4_word_ticks, get_v4_ticks,
        },
    },
};

pub enum AnyPool {
    V3(
        PoolState,
        V3Key,
        V3Contract,
    ),
    V4(
        PoolState,
        V4Key,
        V4Contract,
    ),
}

impl AnyPool {
    pub async fn trade(&mut self, amount_in: U256) {
        match self {
            AnyPool::V3(
                pool_state,
                v3_key,
                v3_pool_instance,
            ) => todo!(),
            AnyPool::V4(
                pool_state,
                pool_key,
                state_view_instance,
            ) => todo!(),
        }
    }

    pub async fn create_v4(
        pool_key: PoolKey,
        provider_url: Url,
        contract_addr: Address,
    ) -> Result<AnyPool, alloy_contract::Error> {
        let provider = ProviderBuilder::new()
            .connect_http(provider_url);
        let contract = StateViewInstance::new(
            contract_addr,
            provider,
        );
        let encoded_key = pool_key.abi_encode();
        let pool_id = keccak256(encoded_key);
        let slot0 = contract
            .getSlot0(pool_id)
            .call()
            .await?;
        let liquidity = contract
            .getLiquidity(pool_id)
            .call()
            .await?;
        let normalized_tick = bitmap_math::normalize_tick(
            slot0.tick,
            pool_key.tickSpacing,
        );
        let word_index =
            bitmap_math::word_index(normalized_tick);
        let bitmap = contract
            .getTickBitmap(
                pool_id, word_index,
            )
            .call()
            .await?;

        let mut hashmap = HashMap::new();
        hashmap.insert(
            word_index, bitmap,
        );
        let active_ticks = fetch_v4_word_ticks(
            &contract,
            bitmap,
            PoolId::from_underlying(pool_id),
            word_index,
            pool_key.tickSpacing,
        )
        .await;
        println!(
            "v4 loaded: {}",
            slot0.sqrtPriceX96
        );

        let state = PoolState {
            current_tick: slot0.tick,
            active_ticks: active_ticks,
            bitmap: hashmap,
            liquidity: U256::from(liquidity),
            x96price: U256::from(slot0.sqrtPriceX96),
        };
        let any_pool = AnyPool::V4(
            state, pool_key, contract,
        );

        Ok(any_pool)
    }
    pub async fn create_v3(
        provider_url: Url,
        addr: Address,
    ) -> Result<AnyPool, alloy_contract::Error> {
        let provider = ProviderBuilder::new()
            .connect_http(provider_url);
        let contract = V3PoolInstance::new(
            addr, provider,
        );
        let token_0 = contract
            .token0()
            .call()
            .await?;
        let token_1 = contract
            .token1()
            .call()
            .await?;
        let slot0 = contract
            .slot0()
            .call()
            .await?;
        let fee = contract
            .fee()
            .call()
            .await?;
        let tick_spacing = contract
            .tickSpacing()
            .call()
            .await?;
        let normalized_tick = bitmap_math::normalize_tick(
            slot0.tick,
            tick_spacing,
        );

        let liquidity = contract
            .liquidity()
            .call()
            .await?;
        let word_index =
            bitmap_math::word_index(normalized_tick);
        let bitmap = contract
            .tickBitmap(word_index)
            .call()
            .await?;
        let mut hashmap = HashMap::<i16, U256>::new();
        hashmap.insert(
            word_index, bitmap,
        );
        let active_ticks = fetch_v3_word_ticks(
            contract.clone(),
            bitmap,
            word_index,
            tick_spacing,
            5_u8,
            6_u64,
        )
        .await;

        println!(
            "v3 price: {}",
            slot0.sqrtPriceX96
        );
        let config = V3Key {
            address: addr,
            token0: token_0,
            token1: token_1,
            fee,
            tick_spacing,
        };
        let state = PoolState {
            current_tick: slot0.tick,
            active_ticks: active_ticks,
            bitmap: hashmap,
            liquidity: U256::from(liquidity),
            x96price: U256::from(slot0.sqrtPriceX96),
        };
        let any_pool = AnyPool::V3(
            state, config, contract,
        );

        Ok(any_pool)
    }
    pub async fn get_ticks(
        &self,
        ticks: Vec<I24>,
    ) -> (
        Vec<Tick>,
        Vec<usize>,
    ) {
        match self {
            AnyPool::V3(_, _, contract) => {
                let (n, f) = get_v3_ticks(
                    contract.clone(),
                    ticks,
                )
                .await;
                return (n, f);
            }
            AnyPool::V4(_, key, contract) => {
                let key = keccak256(key.abi_encode());
                let (n, f) = get_v4_ticks(
                    contract.clone(),
                    ticks,
                    key,
                )
                .await;
                return (n, f);
            }
        }
    }
}
