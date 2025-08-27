use std::fmt::Debug;

use alloy::primitives::{
    aliases::{I24, U24},
    Address,
};
use alloy_provider::Provider;
use serde::{Deserialize, Serialize};

use crate::{sol_types::PoolKey, v2_pool::V2Pool, v3_pool::V3Pool, v4_pool::V4Pool};

pub enum AnyPool<P: Provider + Clone> {
    V2(V2Pool<P>),
    V3(V3Pool<P>),
    V4(V4Pool<P>),
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct V4Key {
    pub currency0: Address,
    pub currency1: Address,
    pub fee: U24,
    pub tickSpacing: I24,
    pub hooks: Address,
}

impl Into<PoolKey> for V4Key {
    fn into(self) -> PoolKey {
        PoolKey {
            currency0: self.currency0,
            currency1: self.currency1,
            fee: self.fee,
            tickSpacing: self.tickSpacing,
            hooks: self.hooks,
        }
    }
}

impl From<PoolKey> for V4Key {
    fn from(value: PoolKey) -> Self {
        Self {
            currency0: value.currency0,
            currency1: value.currency1,
            fee: value.fee,
            tickSpacing: value.tickSpacing,
            hooks: value.hooks,
        }
    }
}
