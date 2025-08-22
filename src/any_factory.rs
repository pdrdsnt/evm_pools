use alloy::{
    primitives::{
        aliases::{I24, U24},
        Address,
    },
    providers::Provider,
    signers::k256::elliptic_curve::pkcs8::der::Encode,
};

use crate::{
    any_pool::AnyPool,
    sol_types::{
        IUniswapV2Factory::IUniswapV2FactoryInstance,
        IUniswapV2Pair::IUniswapV2PairInstance,
        IUniswapV3Factory::IUniswapV3FactoryInstance, PoolKey,
        StateView::StateViewInstance,
    },
};

#[derive(Debug)]
pub enum AnyFactory<P: Provider> {
    V2(IUniswapV2FactoryInstance<P>),
    V3(IUniswapV3FactoryInstance<P>, V3FactoryConfig),
    V4(StateViewInstance<P>, V4FactoryConfig),
}

const COMMON_FEES: [u32; 10] = [
    100, 250, 500, 1000, 1500, 2000, 2500, 3000, 5000, 10000,
];

const COMMON_TICK_SPACINGS: [i32; 20] = [
    1, 2, 5, 10, 15, 20, 25, 30, 40, 50, 60, 70, 100, 110, 120, 130, 140, 160, 200, 240,
];

#[derive(Debug)]
pub struct V3FactoryConfig {
    pub checked_fees: Vec<Result<U24, ()>>,
}

#[derive(Debug)]
pub struct V4FactoryConfig {
    pub checked_fees: Vec<Result<U24, ()>>,
    pub checked_tick_spacing: Vec<Result<I24, ()>>,
}

impl<P: Provider> AnyFactory<P> {
    pub async fn find_pools(
        &self,
        mut token0: Address,
        mut token1: Address,
    ) -> Vec<AnyPool> {
        let mut pools = Vec::new();
        match self {
            AnyFactory::V2(iuniswap_v2_factory_instance) => {
                if token0 > token1 {
                    let _t = *token0;
                    *token0 = *token1;
                    *token1 = _t;
                }
                let provider = iuniswap_v2_factory_instance.provider();
                if let Ok(address) = iuniswap_v2_factory_instance
                    .getPair(token0, token1)
                    .call()
                    .await
                {
                    if let Some(pool) =
                        AnyPool::create_v2_from_address(address, Some(3000), provider)
                            .await
                    {
                        pools.push(pool);
                    }
                }
            }

            AnyFactory::V3(iuniswap_v3_factory_instance, v3_factory_config) => {
                for fee in COMMON_FEES {
                    if let Ok(address) = iuniswap_v3_factory_instance
                        .getPool(token0, token1, U24::from(fee))
                        .call()
                        .await
                    {
                        if let Some(pool) = AnyPool::create_v3_from_address(
                            address,
                            iuniswap_v3_factory_instance.provider(),
                        )
                        .await
                        {
                            pools.push(pool);
                        }
                    }
                }
            }
            AnyFactory::V4(state_view_instance, v4_factory_config) => {
                for fee in COMMON_FEES {
                    for tick_spacing in COMMON_TICK_SPACINGS {
                        let pool_key = PoolKey {
                            currency0: token0,
                            currency1: token1,
                            fee: U24::from(fee),
                            tickSpacing: I24::try_from(tick_spacing).unwrap(),
                            hooks: Address::ZERO,
                        };
                        if let Some(pool) = AnyPool::create_v4_from_address(
                            state_view_instance.address(),
                            key,
                            iuniswap_v3_factory_instance.provider(),
                        )
                        .await
                        {
                            pools.push(pool);
                        }
                    }
                }
            }
        }
        pools
    }
}
