use std::{collections::HashMap, time::Duration};

use alloy::{
    primitives::{Address, U256, aliases::I24, keccak256},
    transports::http::reqwest::Url,
};
use alloy_contract::Error;
use alloy_provider::{
    ProviderBuilder, RootProvider, fillers::FillProvider,
    utils::JoinedRecommendedFillers,
};
use alloy_sol_types::SolValue;
use futures::{StreamExt, stream::FuturesOrdered};
use tokio::time::sleep;

use crate::{
    err::TickError,
    sol_types::{
        PoolId, PoolKey, StateView::StateViewInstance,
        V3Pool::V3PoolInstance,
    },
    tick_math::{self, Tick},
    v3_state::{self, AnyPool, V3State},
};
pub type V3Contract = V3PoolInstance<
    FillProvider<JoinedRecommendedFillers, RootProvider>,
>;
pub type V4Contract = StateViewInstance<
    FillProvider<JoinedRecommendedFillers, RootProvider>,
>;
//unique stae view
pub async fn create_v3(
    provider_url: Url,
    addr: Address,
) -> Result<AnyPool, Error> {
    let provider =
        ProviderBuilder::new().connect_http(provider_url);
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
    let normalized_tick = tick_math::normalize_tick(
        slot0.tick,
        tick_spacing,
    );

    let liquidity = contract
        .liquidity()
        .call()
        .await?;
    let word_index = tick_math::word_index(normalized_tick);
    let bitmap = contract
        .tickBitmap(word_index)
        .call()
        .await?;
    let mut hashmap = HashMap::<i16, U256>::new();
    hashmap.insert(
        word_index, bitmap,
    );
    let active_ticks = AnyPool::fetch_v3_word_ticks(
        contract.clone(),
        bitmap,
        word_index,
        tick_spacing,
    )
    .await;

    println!(
        "v3 price: {}",
        slot0.sqrtPriceX96
    );
    let state = V3State {
        address: addr,
        token0: token_0,
        token1: token_1,
        fee: fee,
        current_tick: slot0.tick,
        active_ticks: active_ticks,
        bitmap: hashmap,
        tick_spacing,
        liquidity: U256::from(liquidity),
        x96price: U256::from(slot0.sqrtPriceX96),
    };
    let any_pool = AnyPool::V3(
        state, contract,
    );

    Ok(any_pool)
}
pub async fn create_v4(
    pool_key: PoolKey,
    provider_url: Url,
    contract_addr: Address,
) -> Result<AnyPool, Error> {
    let provider =
        ProviderBuilder::new().connect_http(provider_url);
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
    let normalized_tick = tick_math::normalize_tick(
        slot0.tick,
        pool_key.tickSpacing,
    );
    let word_index = tick_math::word_index(normalized_tick);
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
    let state = V3State {
        address: contract_addr,
        token0: pool_key.currency0,
        token1: pool_key.currency1,
        fee: pool_key.fee,
        current_tick: slot0.tick,
        active_ticks,
        bitmap: hashmap,
        tick_spacing: pool_key.tickSpacing,
        liquidity: U256::from(liquidity),
        x96price: U256::from(slot0.sqrtPriceX96),
    };
    let any_pool = AnyPool::V4(
        state, contract,
    );
    Ok(any_pool)
}
pub async fn fetch_v4_word_ticks(
    contract: &V4Contract,
    bitmap: U256,
    pool_id: PoolId,
    //these are needed to convert back the local word space to global tick spacing
    word_index: i16,
    tick_spacing: I24,
) -> Vec<Tick> {
    let ticks = tick_math::extract_ticks_from_bitmap(
        bitmap,
        I24::try_from(word_index).unwrap(),
        tick_spacing,
    );

    let mut active_ticks =
        Vec::<Tick>::with_capacity(ticks.len());
    let mut fut_ordered = FuturesOrdered::new();
    for tick in ticks.clone() {
        let c = contract.clone();
        let k = pool_id
            .clone()
            .into_underlying();
        fut_ordered.push_back(
            async move {
                c.getTickInfo(k, tick)
                    .call()
                    .await
            },
        );
    }
    let mut tick_index = 0;
    while let Some(result) = fut_ordered
        .next()
        .await
    {
        let current_tick = &ticks[tick_index].clone();

        tick_index += 1;
        match result {
            Ok(res) => {
                active_ticks.push(Tick {
                    tick: *current_tick,
                    liquidity_net: Some(res.liquidityNet),
                });
            }

            Err(_) => {
                active_ticks.push(Tick {
                    tick: *current_tick,
                    liquidity_net: None,
                });
            }
        }
    }
    active_ticks.sort();
    active_ticks
}
