use alloy::primitives::aliases::I24;
use alloy_provider::{
    Provider, RootProvider, fillers::FillProvider, utils::JoinedRecommendedFillers,
};
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{sol_types::StateView::StateViewInstance, v3_base::states::Tick};

pub type V4Contract =
    StateViewInstance<FillProvider<JoinedRecommendedFillers, RootProvider>>;
//unique stae view

pub async fn get_v4_ticks<P: Provider + Clone + Send + Sync>(
    contract: StateViewInstance<P>,
    ticks: &Vec<I24>,
    key: alloy::primitives::FixedBytes<32>,
) -> (Vec<Tick>, Vec<usize>) {
    let mut fut_ordered = FuturesOrdered::new();

    let mut active_ticks = Vec::<Tick>::with_capacity(ticks.len());

    let mut fail = Vec::new();

    for tick in ticks {
        let c = contract.clone();
        let tick = *tick;
        fut_ordered.push_back(async move { c.getTickInfo(key, tick).call().await });
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
