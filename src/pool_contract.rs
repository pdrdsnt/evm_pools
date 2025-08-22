use alloy_provider::Provider;

use crate::sol_types::{
    IUniswapV2Pair::IUniswapV2PairInstance, StateView::StateViewInstance,
    V3Pool::V3PoolInstance,
};

pub enum PoolContract<P: Provider> {
    V2(IUniswapV2PairInstance<P>),
    V3(V3PoolInstance<P>),
    V4(StateViewInstance<P>),
}

impl<P: Provider> PoolContract<P> {
    async fn get_ticks(&self) {}
}
