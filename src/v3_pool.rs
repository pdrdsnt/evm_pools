use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    task::Poll,
};

use alloy::{
    primitives::{aliases::I24, U160, U256},
    rpc::{
        client::ClientBuilder,
        types::{Bundle, EthCallResponse, TransactionRequest},
    },
};
use alloy_contract::{CallBuilder, EthCall};
use alloy_provider::{Caller, EthCallMany, MulticallBuilder, Provider, ProviderBuilder};
use alloy_sol_types::SolCall;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;

use crate::{
    any_pool::{AnyPool, V4Key},
    any_trade::UniTrade,
    pool::{ConcentratedLiquidity, UniPool, UniTickCall},
    sol_types::{
        StateView::getTickLiquidityCall,
        V3Pool::{liquidityCall, slot0Call, ticksCall, V3PoolInstance},
    },
    v3_base::{
        bitmap_math,
        ticks::{Tick, Ticks},
        v3_state::V3State,
    },
};
pub struct V3Pool<P: Provider> {
    pub key: V4Key,
    pub state: V3State,
    pub contract: V3PoolInstance<P>,
}

impl<P: Provider> V3Pool<P> {
    pub fn new(address: alloy::primitives::Address, provider: P) -> Self {
        let contract = V3PoolInstance::new(address, provider);

        let state = V3State::default(I24::ONE);
        let key = V4Key::default();

        Self {
            key,
            state,
            contract,
        }
    }
}

impl<P: Provider> UniPool for V3Pool<P> {
    fn trade(
        &mut self,
        amount: alloy::primitives::U256,
        from0: bool,
    ) -> Result<UniTrade, crate::err::TradeError> {
        let state = &mut self.state;

        let fee = self.key.fee;

        match crate::v3_base::trade_math::trade_start(&state, &fee, amount, from0) {
            Ok(ok) => Ok(UniTrade::V3(ok)),
            Err(err) => Err(err),
        }
    }

    async fn sync(&mut self) -> Result<(), ()> {
        let state = &mut self.state;
        let contract = &mut self.contract;
        if let Ok(liquidity) = contract.liquidity().call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity == 0 {
                return Err(());
            }
        }

        if let Ok(slot0) = contract.slot0().call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                return Ok(());
            }
        }
        Err(())
    }

    fn create_sync_call(&self) -> Vec<TransactionRequest> {
        let mut calls = Vec::new();

        let contract = &self.contract;
        let lreq = contract.liquidity().into_transaction_request();
        let sreq = contract.slot0().into_transaction_request();

        println!("=============");
        println!("liquidity tx: {:?}", &lreq);
        println!("------------------");
        println!("slot0 tx: {:?}", &sreq);
        println!("=============");

        calls.push(lreq);
        calls.push(sreq);

        calls
    }

    fn decode_sync_result(&mut self, response: Vec<EthCallResponse>) -> Result<(), ()> {
        if response.is_empty() {
            return Err(());
        }

        let liquidity_response = &response[0];
        println!("liquidity respons {:?}", liquidity_response);
        let slot0_response = &response[1];
        println!("slot0 respons {:?}", slot0_response);

        if let Some(bytes) = &liquidity_response.value {
            let r = liquidityCall::abi_decode_returns(bytes.to_vec().as_slice()).unwrap();
            self.state.liquidity = U256::from(r);
        } else {
            return Err(());
        }

        if let Some(bytes) = &slot0_response.value {
            let r = slot0Call::abi_decode_returns(bytes.to_vec().as_slice()).unwrap();

            println!("v3 slot 0 tick: {:?}", r.tick);
            println!("v3 slot 0 liquidity: {:?}", r.sqrtPriceX96);

            self.state.x96price = U256::from(r.sqrtPriceX96);
            self.state.tick = r.tick;
        } else {
            return Err(());
        }
        Ok(())
    }

    fn get_a(&self) -> &alloy::primitives::Address {
        &self.key.currency0
    }

    fn get_b(&self) -> &alloy::primitives::Address {
        &self.key.currency1
    }

    fn get_price(&self) -> U256 {
        self.state.x96price
    }

    fn get_liquidity(&self) -> U256 {
        self.state.liquidity
    }
}

impl<P: Provider> ConcentratedLiquidity for V3Pool<P> {
    fn get_tick_spacing(&self) -> I24 {
        self.key.tickspacing
    }
    fn create_tick_call(&self, tick: I24) -> TransactionRequest {
        let call = self
            .contract
            .ticks(tick)
            .into_transaction_request();

        call
    }

    async fn request_tick(&self, tick: I24) -> Result<Tick, ()> {
        let contract = &self.contract;
        if let Ok(res) = contract.ticks(tick).call().await {
            return Ok(Tick {
                tick,
                liquidity_net: Some(res.liquidityNet),
            });
        }
        Err(())
    }

    async fn request_word(&self, pos: i16) -> Result<U256, ()> {
        let contract = &self.contract;
        if let Ok(res) = contract.tickBitmap(pos).call().await {
            return Ok(res);
        }
        Err(())
    }

    fn create_word_call(&self, pos: i16) -> TransactionRequest {
        let call = self
            .contract
            .tickBitmap(pos)
            .into_transaction_request();

        call
    }

    fn get_mut_ticks(&mut self) -> &mut Ticks {
        &mut self.state.ticks
    }
}
impl<P: Provider + Clone> Into<AnyPool<P>> for V3Pool<P> {
    fn into(self) -> AnyPool<P> {
        AnyPool::V3(self)
    }
}
