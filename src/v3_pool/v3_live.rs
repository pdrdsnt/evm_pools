use std::{collections::HashMap, time::Duration};

use alloy::{
    primitives::{Address, U256, aliases::I24},
    transports::http::reqwest::Url,
};
use alloy_provider::{
    ProviderBuilder, RootProvider, fillers::FillProvider,
    utils::JoinedRecommendedFillers,
};
use futures::{StreamExt, stream::FuturesOrdered};
use tokio::time::sleep;

use crate::{
    any_pool::AnyPool,
    sol_types::V3Pool::V3PoolInstance,
    tick_math::{self, Tick},
    v3_pool::v3_state::V3State,
};

pub type V3Contract = V3PoolInstance<
    FillProvider<JoinedRecommendedFillers, RootProvider>,
>;

pub async fn create_v3(
    provider_url: Url,
    addr: Address,
) -> Result<AnyPool, alloy_contract::Error> {
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
pub async fn fetch_v3_word_ticks(
    contract: V3Contract,

    bitmap: U256,
    word_index: i16,
    tick_spacing: I24,

    max_retries: u8,
    wait_time: u64,
) -> Vec<Tick> {
    let ticks = tick_math::extract_ticks_from_bitmap(
        bitmap,
        I24::try_from(word_index).unwrap(),
        tick_spacing,
    );

    println!(
        "fetch_v3_word_ticks → extracted {} ticks from bitmap at word_index = {}",
        ticks.len(),
        word_index
    );

    let (mut active_ticks, mut fails) = get_v3_ticks(
        contract.clone(),
        ticks,
    )
    .await;
    let mut tries = 0;
    let mut idx_remap = fails.clone();
    while (fails.len() > 0) && tries < max_retries {
        println!("====== retry =====");
        let tks: Vec<I24> = fails
            .iter()
            .map(|t| active_ticks[*t].tick)
            .collect();

        let (mut new_ticks, mut new_fails) = get_v3_ticks(
            contract.clone(),
            tks,
        )
        .await;
        let mut fail_idx = 0;
        let mut sucess_idx = 0;
        let mut new_remap = Vec::new();
        for (idx, new_tick) in new_ticks
            .iter()
            .enumerate()
        {
            if new_tick
                .liquidity_net
                .is_none()
            {
                new_remap.push(idx_remap[idx]);
                fail_idx += 1;
            } else {
                println!(
                    "✅ tick={}, index {}",
                    new_tick.tick, idx_remap[sucess_idx],
                );
                active_ticks[idx_remap[sucess_idx]]
                    .liquidity_net = new_tick.liquidity_net;
                sucess_idx += 1;
            }
        }
        idx_remap = new_remap;
        fails = new_fails;
        tries += 1;
        if !fails.is_empty() && tries < max_retries {
            println!(
                "Waiting {} second before retry {}/{}",
                wait_time, tries, max_retries
            );
            sleep(Duration::from_secs(wait_time)).await;
        }
    }

    active_ticks
}
pub async fn get_v3_ticks(
    contract: V3Contract,
    ticks: Vec<I24>,
) -> (
    Vec<Tick>,
    Vec<usize>,
) {
    let mut active_ticks =
        Vec::<Tick>::with_capacity(ticks.len());

    let mut fail = Vec::new();

    let mut fut_ordered = FuturesOrdered::new();

    for tick in &ticks {
        let c = contract.clone();
        fut_ordered.push_back(
            async move {
                c.ticks(*tick)
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
        let current_tick = &ticks[tick_index];
        tick_index += 1;

        match result {
            Ok(res) => {
                println!(
                    "✅ tick={} → liquidity_net = {}",
                    current_tick, res.liquidityNet
                );
                active_ticks.push(Tick {
                    tick: *current_tick,
                    liquidity_net: Some(res.liquidityNet),
                });
            }
            Err(e) => {
                println!(
                    "❌ tick={} → error: {:?}",
                    current_tick, e
                );
                fail.push(tick_index - 1);
                active_ticks.push(Tick {
                    tick: *current_tick,
                    liquidity_net: None,
                });
            }
        }
    }

    (
        active_ticks,
        fail,
    )
}
