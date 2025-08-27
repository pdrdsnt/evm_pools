use std::{future::Future, marker::PhantomData, pin::Pin};

use alloy::{
    primitives::{aliases::I24, U256},
    rpc::types::{Bundle, TransactionRequest},
};
use alloy_contract::EthCall;

use crate::{
    any_trade::UniTrade,
    sol_types::{StateView::getTickInfoCall, V3Pool::ticksCall},
    v3_base::ticks::Tick,
};

pub trait UniPool {
    fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) -> Result<UniTrade, crate::err::TradeError>;

    async fn sync(&mut self) -> Result<(), ()>;
    fn create_sync_call(&self) -> Vec<TransactionRequest>;

    // async fn get_a(&self) -> &Address;
    // async fn get_b(&self) -> &Address;
    // async fn get_price(&self) -> &U256;
    // async fn get_liquidity(&self) -> &U256;
}

pub trait ConcentratedLiquidity: UniPool {
    async fn request_tick(&self, tick: I24) -> Result<Tick, ()>;
    fn create_tick_call(&self, tick: I24) -> TransactionRequest;
    async fn request_word(&self, pos: i16) -> Result<U256, ()>;
    fn create_word_call(&self, pos: i16) -> TransactionRequest;
}
pub enum UniTickCall {
    V3(EthCall<'static, PhantomData<ticksCall>, alloy::network::Ethereum>),
    V4(EthCall<'static, PhantomData<getTickInfoCall>, alloy::network::Ethereum>),
}
