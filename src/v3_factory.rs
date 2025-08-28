use alloy::primitives::aliases::U24;
use alloy_provider::Provider;

use crate::{any_pool::AnyPool, sol_types::IUniswapV3Factory::IUniswapV3FactoryInstance};

pub struct V3Factory<P: Provider> {
    pub contract: IUniswapV3FactoryInstance<P>,
    pub fees: Vec<U24>,
    pub pools: AnyPool<P>,
}
