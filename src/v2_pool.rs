use std::{future::Future, pin::Pin};

use crate::{
    any_pool::AnyPool,
    pool::UniPool,
    sol_types::IUniswapV2Pair::{getReservesCall, IUniswapV2PairInstance},
    v2_base::{V2Key, V2State},
};

use alloy::{
    primitives::{aliases::U112, Address, U256},
    rpc::types::{EthCallResponse, TransactionRequest},
};
use alloy_provider::Provider;
use alloy_sol_types::SolCall;
pub struct V2Pool<P: Provider> {
    pub key: V2Key,
    pub state: V2State,
    pub contract: IUniswapV2PairInstance<P>,
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

    fn decode_sync_result(&mut self, response: Vec<EthCallResponse>) -> Result<(), ()> {
        if response.is_empty() {
            return Err(());
        }
        let r = &response[0];
        if let Some(bytes) = &r.value {
            let r =
                getReservesCall::abi_decode_returns(bytes.to_vec().as_slice()).unwrap();

            self.state.reserves0 = U256::from(r.reserve0);
            self.state.reserves1 = U256::from(r.reserve1);
        } else {
            return Err(());
        }

        Ok(())
    }

    fn get_a(&self) -> &Address {
        &self.key.token0
    }

    fn get_b(&self) -> &Address {
        &self.key.token1
    }

    fn get_price(&self) -> U256 {
        self.state
            .reserves1
            .checked_div(self.state.reserves0)
            .unwrap_or_default()
    }

    fn get_liquidity(&self) -> U256 {
        U256::from(self.state.reserves0) + U256::from(self.state.reserves1)
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
        let state = V2State::default();

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
