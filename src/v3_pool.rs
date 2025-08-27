use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    task::Poll,
};

use alloy::{
    primitives::{aliases::I24, U160, U256},
    rpc::{
        client::ClientBuilder,
        types::{Bundle, TransactionRequest},
    },
};
use alloy_contract::{CallBuilder, EthCall};
use alloy_provider::{Caller, EthCallMany, MulticallBuilder, Provider, ProviderBuilder};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;

use crate::{
    any_pool::{AnyPool, V4Key},
    any_trade::UniTrade,
    pool::{ConcentratedLiquidity, UniPool, UniTickCall},
    sol_types::V3Pool::{ticksCall, V3PoolInstance},
    v3_base::{
        bitmap_math,
        ticks::{Tick, Ticks},
        v3_state::V3State,
    },
};
pub struct V3Pool<P: Provider> {
    key: V4Key,
    state: V3State,
    contract: V3PoolInstance<P>,
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
        calls.push(contract.liquidity().into_transaction_request());
        calls.push(contract.slot0().into_transaction_request());

        calls
    }
}

impl<P: Provider> ConcentratedLiquidity for V3Pool<P> {
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
}
impl<P: Provider + Clone> Into<AnyPool<P>> for V3Pool<P> {
    fn into(self) -> AnyPool<P> {
        AnyPool::V3(self)
    }
}
