use alloy::primitives::{aliases::I24, ruint::aliases::U256};
use serde::{Deserialize, Serialize};

use crate::v3_base::{bitmap, v3_state};

use super::{bitmap::BitMap, ticks::Ticks};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct V3State {
    pub tick: I24,
    pub ticks: Ticks,
    pub bitmap: BitMap,
    pub liquidity: U256,
    pub x96price: U256,
}
impl V3State {
    pub fn default(tick_spacing: I24) -> Self {
        let tick = I24::ZERO;
        let ticks = Ticks::new(vec![]);
        let bitmap = BitMap::new(tick_spacing, vec![]);
        let liquidity = U256::ZERO;
        let x96price = U256::ZERO;

        Self {
            tick,
            ticks,
            bitmap,
            liquidity,
            x96price,
        }
    }
}
