use alloy::primitives::{U256, aliases::I24, keccak256};
use alloy_sol_types::SolValue;
use futures::{StreamExt, stream::FuturesOrdered};

use crate::{
    generator::V4Contract,
    sol_types::PoolKey,
    v3_pool::{
        v3_live::{V3Contract, get_v3_ticks},
        v3_state::V3State,
    },
    v3_stuff::states::Tick,
};

pub enum AnyPool {
    V3(
        V3State,
        V3Contract,
    ),
    V4(
        V3State,
        V4Contract,
    ),
}

impl AnyPool {
    pub async fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) {
        match self {
            AnyPool::V3(v3_state, v3_pool_instance) => {
                todo!()
            }
            AnyPool::V4(v3_state, state_view_instance) => {
                todo!()
            }
        }
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
                        liquidity_net: Some(
                            res.liquidityNet,
                        ),
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
    pub async fn get_ticks(
        &self,
        ticks: Vec<I24>,
    ) -> (
        Vec<Tick>,
        Vec<usize>,
    ) {
        let mut active_ticks =
            Vec::<Tick>::with_capacity(ticks.len());

        let mut fail = Vec::new();

        match self {
            AnyPool::V3(_, contract) => {
                let (n, f) = get_v3_ticks(
                    contract.clone(),
                    ticks,
                )
                .await;
                active_ticks = n;
                fail = f;
            }
            AnyPool::V4(state, contract) => {
                let key = keccak256(
                    PoolKey::from(state).abi_encode(),
                );
                let (n, f) = Self::get_v4_ticks(
                    contract.clone(),
                    ticks,
                    key,
                )
                .await;
                active_ticks = n;
                fail = f;
            }
        }
        (
            active_ticks,
            fail,
        )
    }
}
