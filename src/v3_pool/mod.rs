use std::sync::Arc;
use alloy::{
    primitives::{Address, aliases::{I24, U24}},
    providers::Provider,
};
use crate::{sol_types::V3Pool::V3PoolInstance, v3_base::states::Tick};

pub struct V3Key {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub tick_spacing: I24,
}

pub type V3Caller<P> = V3PoolInstance<P>;

pub async fn get_v3_ticks<P: Provider>(
    contract: V3Caller<P>,
    ticks: &Vec<I24>,
) -> (Vec<Tick>, Vec<usize>) {
    let mut result = Vec::with_capacity(ticks.len());
    let mut failed_ticks = Vec::new();
    for (i, tick) in ticks.iter().enumerate() {
        if let Ok(tick_return) = contract.ticks(*tick).call().await {
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