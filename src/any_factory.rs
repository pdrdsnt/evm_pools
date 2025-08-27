use alloy::providers::Provider;

use crate::sol_types::{
    IUniswapV2Factory::IUniswapV2FactoryInstance,
    IUniswapV3Factory::IUniswapV3FactoryInstance, StateView::StateViewInstance,
};

#[derive(Debug)]
pub enum AnyFactory<P: Provider + Clone> {
    V2(IUniswapV2FactoryInstance<P>),
    V3(IUniswapV3FactoryInstance<P>),
    V4(StateViewInstance<P>),
}

pub const COMMON_FEES: [u32; 10] = [
    100, 250, 500, 1000, 1500, 2000, 2500, 3000, 5000, 10000,
];

pub const COMMON_TICK_SPACINGS: [i32; 20] = [
    1, 2, 5, 10, 15, 20, 25, 30, 40, 50, 60, 70, 100, 110, 120, 130, 140, 160, 200, 240,
];
