use std::collections::HashMap;

use alloy::primitives::{
    Address, U256,
    aliases::{I24, U24},
};

use crate::{
    generator::{
        V3Contract, V4Contract,
    },
    sol_types::{
        self,
        V3Pool::{
            V3PoolInstance, slot0Call,
            slot0Return,
        },
    },
    tick_math::{Tick, trade},
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

#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
)]
pub struct Trade {
    pub fee: U24,
    pub token0: Address,
    pub token1: Address,
    pub pool: Address,
    pub from0: bool,
    pub amount_in: U256,
    pub amount_out: U256,
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
            AnyPool::V3(
                v3_state,
                v3_pool_instance,
            ) => todo!(),
            AnyPool::V4(
                v3_state,
                state_view_instance,
            ) => todo!(),
        }
    }
    pub async fn v3_trade(
        state: V3State,
        contract: V3Contract,
    ) {
    }
}
