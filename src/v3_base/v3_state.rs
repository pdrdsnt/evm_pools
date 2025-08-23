use alloy::primitives::{aliases::I24, ruint::aliases::U256};
use alloy_provider::Provider;
use serde::{Deserialize, Serialize};

use crate::sol_types::V3Pool::V3PoolInstance;

use super::{bitmap::BitMap, ticks::Ticks};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct V3State {
    pub tick: I24,
    pub ticks: Ticks,
    pub bitmap: BitMap,
    pub liquidity: U256,
    pub x96price: U256,
}
impl V3State {
    pub async fn sync_v3<P: Provider>(&mut self, contract: V3PoolInstance<P>) {}
}
