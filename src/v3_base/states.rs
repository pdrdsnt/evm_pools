use std::collections::HashMap;

use alloy::primitives::{
    Address, U256,
    aliases::{I24, U24},
};

use crate::v3_base::{bitmap::BitMap, ticks::Ticks};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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
    pub amount_out: U256,
    pub liquidity: U256,
    pub x96price: U256,
    pub tick: I24,
    pub remaining: U256,
    pub from0: bool,
    pub step: TradeStep,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct TradeStep {
    pub amount_possible: U256,
    pub next_tick: Tick,
    pub current_tick_index: usize,
    pub next_tick_index: usize,
    pub next_price: U256,
    pub delta: U256,
}

pub struct V3State {
    pub current_tick: I24,
    pub ticks: Ticks,
    pub bitmap: BitMap,
    pub liquidity: U256,
    pub x96price: U256,
}
