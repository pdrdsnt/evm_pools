use crate::{v2_base::V2Trade, v3_base::states::TradeState};

pub enum UniTrade {
    V2(V2Trade),
    V3(TradeState),
}

impl From<TradeState> for UniTrade {
    fn from(value: TradeState) -> Self {
        Self::V3(value)
    }
}

impl From<V2Trade> for UniTrade {
    fn from(value: V2Trade) -> Self {
        Self::V2(value)
    }
}
