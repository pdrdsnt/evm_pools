use std::{future::Future, pin::Pin};

use crate::{
    any_pool::AnyPool,
    pool::UniPool,
    sol_types::IUniswapV2Pair::IUniswapV2PairInstance,
    v2_base::{V2Key, V2State},
};

use alloy::{
    primitives::{aliases::U112, Address, U256},
    rpc::types::{Bundle, TransactionRequest},
};
use alloy_provider::Provider;
pub struct V2Pool<P: Provider> {
    key: V2Key,
    state: V2State,
    contract: IUniswapV2PairInstance<P>,
}

impl<P: Provider> UniPool for V2Pool<P> {
    fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) -> Result<crate::any_trade::UniTrade, crate::err::TradeError> {
        let state = &mut self.state;
        let trade = state.trade(amount, self.key.fee, from0);

        if let Some(result) = trade {
            return Ok(crate::any_trade::UniTrade::V2(result));
        };

        Err(crate::err::TradeError::V2)
    }

    async fn sync(&mut self) -> Result<(), ()> {
        let state = &mut self.state;
        let contract = &self.contract;

        if let Ok(liquidity) = contract.getReserves().call().await {
            if liquidity.reserve0 == U112::ZERO {
                return Err(());
            }
            state.reserves0 = U256::from(liquidity.reserve0);
            state.reserves1 = U256::from(liquidity.reserve1);
        } else {
            return Err(());
        }

        Ok(())
    }

    fn create_sync_call(&self) -> Vec<TransactionRequest> {
        let contract = &self.contract;
        vec![contract.getReserves().into_transaction_request()]
    }
}

impl<P: Provider> V2Pool<P> {
    pub async fn create_v2_from_address(
        //v2 receives a fee since is hard coded,
        //we need to match know factories with fees before calling
        //or passing the list here and calling factory()
        //or just using the uniswap default 3000
        addr: Address,
        fee: Option<u32>,
        provider: P,
    ) -> Option<V2Pool<P>> {
        let contract = IUniswapV2PairInstance::new(addr, provider);
        let mut state = V2State::default();
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
        let mut key = V2Key {
            fee: 3000,
            address: addr,
            token0: Address::ZERO,
            token1: Address::ZERO,
        };

        if let Ok(t0) = contract.token0().call().await {
            key.token0 = t0;
        }
        if let Ok(t1) = contract.token1().call().await {
            key.token1 = t1;
        }

        if let Some(f) = fee {
            key.fee = f;
        }

        Some(Self {
            key,
            state,
            contract,
        })
    }
}
impl<P: Provider + Clone> Into<AnyPool<P>> for V2Pool<P> {
    fn into(self) -> AnyPool<P> {
        AnyPool::V2(self)
    }
}
