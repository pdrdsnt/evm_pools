use std::{collections::HashMap, time::SystemTime};

use alloy::primitives::{
    aliases::{I24, U24},
    Address,
};
use alloy_provider::Provider;

use crate::{
    any_pool::V4Key,
    sol_types::{PoolKey, StateView::StateViewInstance},
};

pub struct V4Factory<P: Provider> {
    pub contract: StateViewInstance<P>,
    pub pools: Vec<PoolKey>,
}
