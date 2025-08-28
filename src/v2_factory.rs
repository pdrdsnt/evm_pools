use alloy::primitives::aliases::U24;
use alloy_provider::Provider;

use crate::sol_types::IUniswapV2Pair::IUniswapV2PairInstance;

pub struct V2Factory<P: Provider> {
    pub contract: IUniswapV2PairInstance<P>,
    pub fee: U24,
}
