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
