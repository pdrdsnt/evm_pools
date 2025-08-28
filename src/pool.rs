use std::{future::Future, marker::PhantomData, pin::Pin};

use alloy::{
    primitives::{aliases::I24, Address, U256},
    rpc::types::{Bundle, EthCallResponse, TransactionRequest},
};
use alloy_contract::EthCall;
use futures::future::join_all;

use crate::{
    any_trade::UniTrade,
    sol_types::{StateView::getTickInfoCall, V3Pool::ticksCall},
    v3_base::{
        bitmap_math, tick_math,
        ticks::{Tick, Ticks},
    },
};

pub trait UniPool {
    fn trade(
        &mut self,
        amount: U256,
        from0: bool,
    ) -> Result<UniTrade, crate::err::TradeError>;

    async fn sync(&mut self) -> Result<(), ()>;
    fn create_sync_call(&self) -> Vec<TransactionRequest>;
    fn decode_sync_result(&mut self, responses: Vec<EthCallResponse>) -> Result<(), ()>;

    fn get_a(&self) -> &Address;
    fn get_b(&self) -> &Address;
    fn get_price(&self) -> U256;
    fn get_liquidity(&self) -> U256;
}

pub trait ConcentratedLiquidity: UniPool {
    async fn sync_ticks(&mut self) -> Result<(), ()> {
        let Some(tick) = tick_math::tick_from_price(self.get_price()) else {
            return Err(());
        };
        let tick_spacing = self.get_tick_spacing();
        let normalized_tick = bitmap_math::normalize_tick(tick, tick_spacing);
        let word_pos = bitmap_math::word_index(normalized_tick);
        let words_pos = [
            word_pos - 1,
            word_pos,
            word_pos + 1,
        ];
        let mut ticks = Vec::<I24>::new();
        for pos in words_pos {
            match self.request_word(pos).await {
                Ok(w) => {
                    let mut t =
                        bitmap_math::extract_ticks_from_bitmap(w, pos, tick_spacing);
                    ticks.append(&mut t);
                }
                Err(_) => (),
            }
        }
        let mut futs = Vec::new();
        for t in ticks {
            futs.push(self.request_tick(t));
        }
        let mut tks = Vec::<Tick>::new();
        for r in join_all(futs).await {
            match r {
                Ok(ok) => tks.push(ok),
                Err(_) => (),
            }
        }

        self.get_mut_ticks().insert_ticks(tks);

        Ok(())
    }

    fn get_tick_spacing(&self) -> I24;
    fn get_mut_ticks(&mut self) -> &mut Ticks;
    async fn request_tick(&self, tick: I24) -> Result<Tick, ()>;
    fn create_tick_call(&self, tick: I24) -> TransactionRequest;
    async fn request_word(&self, pos: i16) -> Result<U256, ()>;
    fn create_word_call(&self, pos: i16) -> TransactionRequest;
}
pub enum UniTickCall {
    V3(EthCall<'static, PhantomData<ticksCall>, alloy::network::Ethereum>),
    V4(EthCall<'static, PhantomData<getTickInfoCall>, alloy::network::Ethereum>),
}
