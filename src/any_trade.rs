use crate::{v2_pool::V2Trade, v3_base::states::TradeState};

#[derive(Debug)]
pub enum AnyTrade {
    V2(V2Trade),
    V3(TradeState),
    V4(TradeState),
}
