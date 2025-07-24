use std::collections::HashMap;

use alloy::primitives::{
    Address, U256,
    aliases::{I24, U24},
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Tick {
    pub tick: I24,
    pub liquidity_net: Option<i128>,
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
    pub fee_amount: U256,
    pub amount_in: U256,
    pub liquidity: U256,
    pub x96price: U256,
    pub tick: I24,
    pub remaining: U256,
}

pub struct PoolState {
    pub current_tick: I24,
    pub active_ticks: Vec<Tick>,
    pub bitmap: HashMap<i16, U256>,
    pub liquidity: U256,
    pub x96price: U256,
}
