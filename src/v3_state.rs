use std::{collections::HashMap, pin::Pin, time::Duration};

use alloy::{
    primitives::{
        Address, U256,
        aliases::{I24, U24},
        keccak256,
    },
    rpc::types::state,
};
use alloy_contract::Error;
use alloy_sol_types::SolValue;
use futures::{
    FutureExt, StreamExt, stream::FuturesOrdered,
};
use tokio::time::sleep;

use crate::{
    generator::{V3Contract, V4Contract},
    sol_types::{
        self, PoolKey,
        StateView::{self, StateViewInstance},
        V3Pool::{V3PoolInstance, slot0Call, slot0Return},
    },
    tick_math::{self, Tick, trade},
    v3_state,
};

pub struct V3State {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub current_tick: I24,
    pub active_ticks: Vec<Tick>,
    pub bitmap: HashMap<i16, U256>,
    pub tick_spacing: I24,
    pub liquidity: U256,
    pub x96price: U256,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TradeReceipt {
    pub fee: U24,
    pub fee_amount: U256,
    pub token0: Address,
    pub token1: Address,
    pub pool: Address,
    pub from0: bool,
    pub amount_in: U256,
    pub amount_out: U256,
}

#[derive(Debug, Clone, Copy)]
pub struct TradeState {
    pub fee: U256,
    pub amount_in: U256,
    pub liquidity: U256,
    pub x96price: U256,
    pub tick: I24,
    pub remaining: U256,
}

pub enum AnyPool {
    V3(
        v3_state::V3State,
        V3Contract,
    ),
    V4(
        v3_state::V3State,
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
    pub async fn fetch_v3_word_ticks(
        contract: V3Contract,
        bitmap: U256,
        word_index: i16,
        tick_spacing: I24,
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

        let (mut active_ticks, mut fails) =
            AnyPool::get_v3_ticks(
                contract.clone(),
                ticks,
            )
            .await;
        let MAX_RETRY = 4;
        let RETRY_WAIT_SECONDS = 5;
        let mut tries = 0;
        let mut idx_remap = fails.clone();
        while (fails.len() > 0) && tries < MAX_RETRY {
            println!("====== retry =====");
            let tks: Vec<I24> = fails
                .iter()
                .map(|t| active_ticks[*t].tick)
                .collect();

            let (mut new_ticks, mut new_fails) =
                AnyPool::get_v3_ticks(
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
                        new_tick.tick,
                        idx_remap[sucess_idx],
                    );
                    active_ticks[idx_remap[sucess_idx]]
                        .liquidity_net =
                        new_tick.liquidity_net;
                    sucess_idx += 1;
                }
            }
            idx_remap = new_remap;
            fails = new_fails;
            tries += 1;
            if !fails.is_empty() && tries < MAX_RETRY {
                println!(
                    "Waiting {} second before retry {}/{}",
                    RETRY_WAIT_SECONDS, tries, MAX_RETRY
                );
                sleep(
                    Duration::from_secs(RETRY_WAIT_SECONDS),
                )
                .await;
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
                let (n, f) = Self::get_v3_ticks(
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

    pub async fn v3_trade(
        state: V3State,
        contract: V3Contract,
    ) {
    }
}
