use alloy::{
    primitives::{aliases::I24, keccak256, B256, U160, U256},
    rpc::types::TransactionRequest,
};
use alloy_provider::Provider;
use alloy_sol_types::SolValue;

use crate::{
    any_pool::{AnyPool, V4Key},
    any_trade::UniTrade,
    pool::{ConcentratedLiquidity, UniPool},
    sol_types::{PoolKey, StateView::StateViewInstance},
    v3_base::{ticks::Tick, v3_state::V3State},
};

pub struct V4Pool<P: Provider> {
    key: V4Key,
    id: B256,
    state: V3State,
    contract: StateViewInstance<P>,
}
impl<P: Provider> V4Pool<P> {
    pub fn new(key: V4Key, contract: StateViewInstance<P>) -> Self {
        let state = V3State::default(I24::ONE);
        let id: PoolKey = key.into();

        Self {
            key,
            id: keccak256(id.abi_encode()),
            state,
            contract,
        }
    }
}

impl<P: Provider> UniPool for V4Pool<P> {
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
        let contract = &self.contract;
        let id = self.id;
        if let Ok(liquidity) = contract.getLiquidity(id).call().await {
            state.liquidity = U256::from(liquidity);
            if liquidity == 0 {
                return Err(());
            }
        };

        if let Ok(slot0) = contract.getSlot0(id).call().await {
            state.x96price = U256::from(slot0.sqrtPriceX96);
            state.tick = slot0.tick;

            if slot0.sqrtPriceX96 != U160::ZERO {
                return Err(());
            }
        };

        Err(())
    }

    fn create_sync_call(&self) -> Vec<TransactionRequest> {
        let mut calls = Vec::new();

        let contract = &self.contract;
        calls.push(
            contract
                .getLiquidity(self.id)
                .into_transaction_request(),
        );
        calls.push(
            contract
                .getSlot0(self.id)
                .into_transaction_request(),
        );
        calls
    }
}

impl<P: Provider> ConcentratedLiquidity for V4Pool<P> {
    fn create_tick_call(&self, tick: I24) -> TransactionRequest {
        let call = self
            .contract
            .getTickInfo(self.id, tick)
            .into_transaction_request();
        call
    }

    async fn request_tick(&self, tick: I24) -> Result<Tick, ()> {
        let contract = &self.contract;
        if let Ok(res) = contract.getTickInfo(self.id, tick).call().await {
            return Ok(Tick {
                tick,
                liquidity_net: Some(res.liquidityNet),
            });
        }
        Err(())
    }

    async fn request_word(&self, pos: i16) -> Result<U256, ()> {
        let contract = &self.contract;
        if let Ok(res) = contract.getTickBitmap(self.id, pos).call().await {
            return Ok(res);
        }
        Err(())
    }

    fn create_word_call(&self, pos: i16) -> TransactionRequest {
        let call = self
            .contract
            .getTickBitmap(self.id, pos)
            .into_transaction_request();

        call
    }
}

impl<P: Provider + Clone> Into<AnyPool<P>> for V4Pool<P> {
    fn into(self) -> AnyPool<P> {
        AnyPool::V4(self)
    }
}
