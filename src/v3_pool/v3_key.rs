use std::{collections::HashMap, pin::Pin, time::Duration};

use alloy::primitives::{
    Address, U256,
    aliases::{I24, U24},
};

pub struct V3Key {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub tick_spacing: I24,
}
