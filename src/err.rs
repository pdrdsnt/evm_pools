#[derive(Debug)]
pub enum TickError {
    Overflow(i16),
    Underflow(i16),
}
#[derive(Debug)]
pub enum MathError {
    A,
    B,
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
