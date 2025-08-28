use std::{fmt::Debug, ops::Add};

use alloy::primitives::{
    aliases::{I24, U24},
    Address,
};
use alloy_provider::Provider;
use serde::{Deserialize, Serialize};

use crate::{
    pool::{ConcentratedLiquidity, UniPool},
    sol_types::PoolKey,
    v2_pool::V2Pool,
    v3_pool::V3Pool,
    v4_pool::V4Pool,
};

pub enum AnyPool<P: Provider + Clone> {
    V2(V2Pool<P>),
    V3(V3Pool<P>),
    V4(V4Pool<P>),
}

impl<P: Provider + Clone> AnyPool<P> {
    pub async fn super_sync(&mut self) -> Result<(), ()> {
        match self {
            AnyPool::V2(v2_pool) => v2_pool.sync().await,
            AnyPool::V3(v3_pool) => {
                v3_pool.sync().await;
                v3_pool.sync_ticks().await
            }
            AnyPool::V4(v4_pool) => {
                v4_pool.sync();
                v4_pool.sync_ticks().await
            }
        }
    }
}
impl<P: Provider + Clone> UniPool for AnyPool<P> {
    fn trade(
        &mut self,
        amount: alloy::primitives::U256,
        from0: bool,
    ) -> Result<crate::any_trade::UniTrade, crate::err::TradeError> {
        match self {
            AnyPool::V2(v2_pool) => v2_pool.trade(amount, from0),
            AnyPool::V3(v3_pool) => v3_pool.trade(amount, from0),
            AnyPool::V4(v4_pool) => v4_pool.trade(amount, from0),
        }
    }

    async fn sync(&mut self) -> Result<(), ()> {
        match self {
            AnyPool::V2(v2_pool) => v2_pool.sync().await,
            AnyPool::V3(v3_pool) => v3_pool.sync().await,
            AnyPool::V4(v4_pool) => v4_pool.sync().await,
        }
    }

    fn create_sync_call(&self) -> Vec<alloy::rpc::types::TransactionRequest> {
        match self {
            AnyPool::V2(v2_pool) => v2_pool.create_sync_call(),
            AnyPool::V3(v3_pool) => v3_pool.create_sync_call(),
            AnyPool::V4(v4_pool) => v4_pool.create_sync_call(),
        }
    }

    fn decode_sync_result(
        &mut self,
        responses: Vec<alloy::rpc::types::EthCallResponse>,
    ) -> Result<(), ()> {
        match self {
            Self::V2(v2_pool) => v2_pool.decode_sync_result(responses),
            Self::V3(v3_pool) => v3_pool.decode_sync_result(responses),
            Self::V4(v4_pool) => v4_pool.decode_sync_result(responses),
        }
    }

    fn get_a(&self) -> &Address {
        match self {
            Self::V2(v2_pool) => v2_pool.get_a(),
            Self::V3(v3_pool) => v3_pool.get_a(),
            Self::V4(v4_pool) => v4_pool.get_a(),
        }
    }

    fn get_b(&self) -> &Address {
        match self {
            Self::V2(v2_pool) => v2_pool.get_b(),
            Self::V3(v3_pool) => v3_pool.get_b(),
            Self::V4(v4_pool) => v4_pool.get_b(),
        }
    }

    fn get_price(&self) -> alloy::primitives::U256 {
        match self {
            Self::V2(v2_pool) => v2_pool.get_price(),
            Self::V3(v3_pool) => v3_pool.get_price(),
            Self::V4(v4_pool) => v4_pool.get_price(),
        }
    }

    fn get_liquidity(&self) -> alloy::primitives::U256 {
        match self {
            Self::V2(v2_pool) => v2_pool.get_liquidity(),
            Self::V3(v3_pool) => v3_pool.get_liquidity(),
            Self::V4(v4_pool) => v4_pool.get_liquidity(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct V4Key {
    pub currency0: Address,
    pub currency1: Address,
    pub fee: U24,
    pub tickspacing: I24,
    pub hooks: Address,
}

impl Into<PoolKey> for V4Key {
    fn into(self) -> PoolKey {
        PoolKey {
            currency0: self.currency0,
            currency1: self.currency1,
            fee: self.fee,
            tickSpacing: self.tickspacing,
            hooks: self.hooks,
        }
    }
}

impl From<PoolKey> for V4Key {
    fn from(value: PoolKey) -> Self {
        Self {
            currency0: value.currency0,
            currency1: value.currency1,
            fee: value.fee,
            tickspacing: value.tickSpacing,
            hooks: value.hooks,
        }
    }
}
impl<P: Provider + Clone> Debug for AnyPool<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyPool::V2(v2_pool) => {
                writeln!(
                    f,
                    "v2 pool {} /n reserves0: {} /n reserves1: {}",
                    v2_pool.key.address, v2_pool.state.reserves0, v2_pool.state.reserves1
                )
            }
            AnyPool::V3(v3_pool) => {
                writeln!(
                    f,
                    "v3 pool {} /n liquidity: {} /n price: {} /n ticks: {:?}",
                    v3_pool.contract.address(),
                    v3_pool.state.liquidity,
                    v3_pool.state.x96price,
                    v3_pool.state.ticks,
                )
            }
            AnyPool::V4(v4_pool) => {
                writeln!(
                    f,
                    "v4 pool {} /n liquidity: {} /n price: {} /n ticks: {:?}",
                    v4_pool.contract.address(),
                    v4_pool.state.liquidity,
                    v4_pool.state.x96price,
                    v4_pool.state.ticks,
                )
            }
        }
    }
}
