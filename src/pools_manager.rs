use alloy::primitives::Address;
use alloy_provider::Provider;

use crate::{any_factory::AnyFactory, any_pool::AnyPool, pool_contract::PoolContract};

pub struct PoolsBuilder<P: Provider> {
    provider: P,
    tokens: Address,
    factories: Vec<AnyFactory<P>>,
    pools: Vec<(AnyPool, PoolContract<P>)>,
}

impl<P: Provider> PoolsBuilder<P> {
    pub fn build() {}
}
