use alloy::{
    primitives::{aliases::I24, keccak256, B256, U160, U256},
    rpc::types::{EthCallResponse, TransactionRequest},
};
use alloy_provider::Provider;
use alloy_sol_types::{SolCall, SolValue};

use crate::{
    any_pool::{AnyPool, V4Key},
    any_trade::UniTrade,
    pool::{ConcentratedLiquidity, UniPool},
    sol_types::{
        PoolKey,
        StateView::{getLiquidityCall, getSlot0Call, getTickInfoCall, StateViewInstance},
    },
    v3_base::{
        ticks::{Tick, Ticks},
        v3_state::V3State,
    },
};

pub struct V4Pool<P: Provider> {
    pub key: V4Key,
    pub id: B256,
    pub state: V3State,
    pub contract: StateViewInstance<P>,
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
        let lreq = contract
            .getLiquidity(self.id)
            .into_transaction_request();
        let sreq = contract
            .getSlot0(self.id)
            .into_transaction_request();

        println!("=============");
        println!("v4 liquidity request {:?}", lreq);
        println!("------------------");
        println!("v4 slot0  request {:?}", sreq);
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
        println!("v4 liquidity respons {:?}", liquidity_response);

        let slot0_response = &response[1];

        println!("v4 slot0 respons {:?}", slot0_response);
        if let Some(bytes) = &liquidity_response.value {
            let r =
                getLiquidityCall::abi_decode_returns(bytes.to_vec().as_slice()).unwrap();

            self.state.liquidity = U256::from(r);
        } else {
            return Err(());
        }
        if let Some(bytes) = &slot0_response.value {
            let r = getSlot0Call::abi_decode_returns(bytes.to_vec().as_slice()).unwrap();

            println!("v4 slot 0 tick: {:?}", r.tick);
            println!("v4 slot 0 liquidity: {:?}", r.sqrtPriceX96);

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

impl<P: Provider> ConcentratedLiquidity for V4Pool<P> {
    fn get_tick_spacing(&self) -> I24 {
        self.key.tickspacing
    }
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
    fn get_mut_ticks(&mut self) -> &mut Ticks {
        &mut self.state.ticks
    }
}

impl<P: Provider + Clone> Into<AnyPool<P>> for V4Pool<P> {
    fn into(self) -> AnyPool<P> {
        AnyPool::V4(self)
    }
}
