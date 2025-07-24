use alloy::primitives::{
    U256, aliases::I24,
};

use crate::v3_state::TradeState;

#[derive(Debug)]
pub enum TickError {
    Overflow(TradeState),
    Underflow(TradeState),
    Unavailable(TradeState),
}
#[derive(Debug)]
pub enum MathError {
    A(TradeState),
}

#[derive(Debug)]
pub enum TradeError {
    Tick(TickError),
    Math(MathError),
}
impl From<TickError> for TradeError {
    fn from(value: TickError) -> Self {
        TradeError::Tick(value)
    }
}

impl From<MathError> for TradeError {
    fn from(value: MathError) -> Self {
        TradeError::Math(value)
    }
}
