use std::{collections::HashMap, time::Duration};

use alloy::{
    primitives::aliases::I24, primitives::aliases::U256,
};
use alloy_provider::{
    ProviderBuilder, RootProvider, fillers::FillProvider,
    utils::JoinedRecommendedFillers,
};
use futures::{StreamExt, stream::FuturesOrdered};
use tokio::time::sleep;

use crate::{
    sol_types::V3Pool::V3PoolInstance,
    v3_base::{bitmap_math, states::Tick},
};

pub type V3Contract = V3PoolInstance<
    FillProvider<JoinedRecommendedFillers, RootProvider>,
>;

pub async fn fetch_v3_word_ticks(
    contract: V3Contract,

    bitmap: U256,
    word_index: i16,
    tick_spacing: I24,

    max_retries: u8,
    wait_time: u64,
) -> Vec<Tick> {
    let ticks = bitmap_math::extract_ticks_from_bitmap(
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

        let (new_ticks, new_fails) = get_v3_ticks(
            contract.clone(),
            tks,
        )
        .await;
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
