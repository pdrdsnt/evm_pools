use std::sync::Arc;

use alloy::{
    primitives::{aliases::I24, keccak256, Address, B256},
    providers::Provider,
};

use crate::{
    sol_types::{PoolKey, StateView::StateViewInstance},
    v3_base::states::Tick,
};

pub type V4Key = crate::sol_types::PoolKey;

pub type V4Contract<P> = StateViewInstance<P>;

pub async fn get_v4_ticks<P: Provider>(
    contract: V4Contract<P>,
    ticks: &Vec<I24>,
    pool_id: B256,
) -> (Vec<Tick>, Vec<usize>) {
    let mut result = Vec::with_capacity(ticks.len());
    let mut failed_ticks = Vec::new();
    for (i, tick) in ticks.iter().enumerate() {
        if let Ok(tick_return) = contract.getTickInfo(pool_id, *tick).call().await {
            result.push(Tick {
                tick: *tick,
                liquidity_net: Some(tick_return.liquidityNet),
            });
        } else {
            failed_ticks.push(i);
        }
    }

    (result, failed_ticks)
}