use std::collections::HashMap;

use alloy::primitives::{U256, aliases::I24};
use alloy_provider::{
    RootProvider, fillers::FillProvider,
    utils::JoinedRecommendedFillers,
};
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{
    sol_types::{PoolId, StateView::StateViewInstance},
    v3_base::{bitmap_math, states::Tick},
};

pub type V4Contract = StateViewInstance<
    FillProvider<JoinedRecommendedFillers, RootProvider>,
>;
//unique stae view

pub async fn fetch_v4_word_ticks(
    contract: &V4Contract,
    bitmap: U256,
    pool_id: PoolId,
    //these are needed to convert back the local word space to global tick spacing
    word_index: i16,
    tick_spacing: I24,
) -> Vec<Tick> {
    let ticks = bitmap_math::extract_ticks_from_bitmap(
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
    active_ticks
}
pub async fn get_v4_ticks(
    contract: V4Contract,
    ticks: Vec<I24>,
    key: alloy::primitives::FixedBytes<32>,
) -> (
    Vec<Tick>,
    Vec<usize>,
) {
    let mut fut_ordered = FuturesOrdered::new();

    let mut active_ticks =
        Vec::<Tick>::with_capacity(ticks.len());

    let mut fail = Vec::new();

    for tick in &ticks {
        let c = contract.clone();
        let tick = *tick;
        fut_ordered.push_back(
            async move {
                c.getTickInfo(key, tick)
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
