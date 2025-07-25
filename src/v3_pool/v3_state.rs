use std::{collections::HashMap, pin::Pin, time::Duration};

use alloy::primitives::{
    Address, U256,
    aliases::{I24, U24},
};
use alloy_sol_types::SolValue;
use futures::{
    FutureExt, StreamExt, stream::FuturesOrdered,
};
use tokio::time::sleep;

use crate::{
    sol_types::{
        self, IHooks, PoolKey,
        StateView::{self, StateViewInstance},
        V3Pool::{V3PoolInstance, slot0Call, slot0Return},
    },
    v3_stuff::states::Tick,
};

pub struct V3Key {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub tick_spacing: I24,
}
