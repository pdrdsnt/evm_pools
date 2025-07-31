use alloy::primitives::aliases::I24;
use alloy_provider::{
    RootProvider, fillers::FillProvider, utils::JoinedRecommendedFillers,
};
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{sol_types::V3Pool::V3PoolInstance, v3_base::states::Tick};

pub type V3Contract =
    V3PoolInstance<FillProvider<JoinedRecommendedFillers, RootProvider>>;

pub async fn get_v3_ticks(
    contract: V3Contract,
    ticks: Vec<I24>,
) -> (Vec<Tick>, Vec<usize>) {
    let mut active_ticks = Vec::<Tick>::with_capacity(ticks.len());

    let mut fail = Vec::new();

    let mut fut_ordered = FuturesOrdered::new();

    for tick in &ticks {
        let c = contract.clone();
        fut_ordered.push_back(async move { c.ticks(*tick).call().await });
    }
    let mut tick_index = 0;
    while let Some(result) = fut_ordered.next().await {
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
                println!("❌ tick={} → error: {:?}", current_tick, e);
                fail.push(tick_index - 1);
                active_ticks.push(Tick {
                    tick: *current_tick,
                    liquidity_net: None,
                });
            }
        }
    }

    (active_ticks, fail)
}
