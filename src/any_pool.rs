use std::intrinsics::atomic_singlethreadfence_acqrel;

use alloy::primitives::{
    aliases::{I24, U24},
    keccak256, Address, B256, U160, U256,
};
use alloy_provider::Provider;
use alloy_sol_types::SolValue;
use serde::{Deserialize, Serialize};

use crate::{
    any_trade::AnyTrade,
    pool_contract::PoolContract,
    sol_types::{
        IUniswapV2Pair::IUniswapV2PairInstance, PoolKey, StateView::StateViewInstance,
        V3Pool::V3PoolInstance,
    },
    v2_pool::V2Pool,
    v3_base::{states::V3State, trade_math::trade_start},
};
#[derive(Serialize, Deserialize, Debug)]
pub enum AnyPool {
    V2(V2Pool, Address),
    V3(V3State, V4Key, Address, Address),
    V4(V3State, V4Key, B256, Address),
}

impl AnyPool {
    pub fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) -> Result<AnyTrade, crate::err::TradeError> {
        let state: &V3State;
        let fee: U24;
        match self {
            AnyPool::V2(v2_pool, _) => match v2_pool.trade(amount, from0) {
                Some(ok) => Ok(AnyTrade::V2(ok)),
                None => Err(crate::err::TradeError::V2),
            },
            AnyPool::V3(v3_state, v4_key, _, _) => {
                state = v3_state;
                fee = v4_key.fee;
                match trade_start(&state, &fee, amount, from0) {
                    Ok(ok) => Ok(AnyTrade::V3(ok)),
                    Err(err) => Err(err),
                }
            }
            AnyPool::V4(v3_state, v4_key, _, _) => {
                state = v3_state;
                fee = v4_key.fee;
                match trade_start(&state, &fee, amount, from0) {
                    Ok(ok) => Ok(AnyTrade::V4(ok)),
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub async fn create_v2_from_address<P: Provider>(
        addr: Address,
        fee: Option<u32>,
        provider: P,
    ) -> Option<AnyPool> {
        let contract = IUniswapV2PairInstance::new(addr, provider);
        let mut state = V2Pool::default();
        let mut one_valid_answer = false;

        if let Ok(r) = contract.getReserves().call().await {
            let r0 = U256::from(r.reserve0);
            let r1 = U256::from(r.reserve1);
            if r0 != U256::ZERO && r1 != U256::ZERO {
                one_valid_answer = true;
                state.reserves0 = r0;
                state.reserves1 = r1;
            }
        }

        if !one_valid_answer {
            return None;
        }

        if let Ok(t0) = contract.token0().call().await {
            state.token0 = t0;
        }
        if let Ok(t1) = contract.token1().call().await {
            state.token1 = t1;
        }

        if let Some(f) = fee {
            state.fee = f;
        } else {
            state.fee = 3000;
        }

        Some(Self::V2(state, addr))
    }

    pub async fn sync_v3<P: Provider>(state: &mut V3State, contract: V3PoolInstance<P>) {
        if let Ok(liquidity) = contract.liquidity().call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity != 0 {}
        }
        if let Ok(slot0) = contract.slot0().call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {}
        }
    }

    pub async fn create_v3_from_address<P: Provider>(
        addr: Address,
        provider: P,
    ) -> Option<AnyPool> {
        let contract = V3PoolInstance::new(addr, provider);
        let mut state = V3State::default();
        let mut one_valid_answer = false;

        if let Ok(liquidity) = contract.liquidity().call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity != 0 {
                one_valid_answer = true;
            }
        }

        if let Ok(slot0) = contract.slot0().call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                one_valid_answer = true;
            }
        }

        if !one_valid_answer {
            return None;
        }

        let mut key = V4Key::default();

        if let Ok(fee) = contract.fee().call().await {
            key.fee = fee;
        }
        if let Ok(t0) = contract.token0().call().await {
            key.currency0 = t0;
        }
        if let Ok(t1) = contract.token1().call().await {
            key.currency1 = t1;
        }
        if let Ok(ts) = contract.tickSpacing().call().await {
            key.tickSpacing = ts;
        }
        let mut f = Address::ZERO;
        if let Ok(factory) = contract.factory().call().await {
            f = factory;
        }

        Some(Self::V3(state, key, addr, f))
    }

    pub async fn create_v4_from_key<P: Provider>(
        contract: StateViewInstance<P>,
        key: V4Key,
        provider: P,
    ) -> Option<AnyPool> {
        let mut state = V3State::default();
        let mut one_valid_answer = false;

        let _id: PoolKey = key.into();
        let id = keccak256(_id.abi_encode());
        if let Ok(liquidity) = contract.getLiquidity(id).call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity != 0 {
                one_valid_answer = true;
            }
        }

        if let Ok(slot0) = contract.getSlot0(id).call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                one_valid_answer = true;
            }
        }

        if !one_valid_answer {
            return None;
        }

        Some(Self::V4(state, key, id, addr))
    }

    pub fn build_contract<P: Provider + Clone>(&self, provider: P) -> PoolContract<P> {
        match self {
            AnyPool::V2(pool, _) => {
                PoolContract::V2(IUniswapV2PairInstance::new(pool.address, provider))
            }
            AnyPool::V3(_, _, address, _) => {
                PoolContract::V3(V3PoolInstance::new(*address, provider.clone()))
            }
            AnyPool::V4(_, _, _, address) => {
                PoolContract::V4(StateViewInstance::new(*address, provider.clone()))
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct V4Key {
    currency0: Address,
    currency1: Address,
    fee: U24,
    tickSpacing: I24,
    hooks: Address,
}

impl Into<PoolKey> for V4Key {
    fn into(self) -> PoolKey {
        PoolKey {
            currency0: self.currency0,
            currency1: self.currency1,
            fee: self.fee,
            tickSpacing: self.tickSpacing,
            hooks: self.hooks,
        }
    }
}
