use std::collections::HashMap;

use alloy::{
    primitives::{Address, U256, aliases::I24, keccak256},
    transports::http::reqwest::Url,
};
use alloy_provider::{
    ProviderBuilder, RootProvider, fillers::FillProvider,
    utils::JoinedRecommendedFillers,
};
use alloy_sol_types::SolValue;
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{
    any_pool::AnyPool,
    sol_types::{
        PoolId, PoolKey, StateView::StateViewInstance,
    },
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
    active_ticks
}
