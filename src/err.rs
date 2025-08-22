use crate::v3_base::states::TradeState;
#[derive(Debug, Clone)]
pub enum WordError {
    NotTried,
}

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
    Fetch(alloy_contract::Error),
    Math(MathError),
    V2,
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
impl From<alloy_contract::Error> for TradeError {
    fn from(value: alloy_contract::Error) -> Self {
        TradeError::Fetch(value)
    }
}
