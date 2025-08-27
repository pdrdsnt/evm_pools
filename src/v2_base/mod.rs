use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct V2State {
    pub reserves0: U256,
    pub reserves1: U256,
}

pub struct V2Key {
    pub fee: u32,
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}

impl V2State {
    pub fn trade(&self, amount_in: U256, fee: u32, from0: bool) -> Option<V2Trade> {
        if (from0 && self.reserves0 == U256::ZERO)
            || (!from0 && self.reserves1 == U256::ZERO)
        {
            return None;
        }

        // 2. Get reserves in proper decimal scale
        let (reserve_in, reserve_out) = match from0 {
            true => (self.reserves0, self.reserves1),
            false => (self.reserves1, self.reserves0),
        };

        let sfee = fee / 1000;

        // 3. Apply V2 fee calculation correctly (0.3% fee)
        let amount_in_less_fee = amount_in
            .checked_mul(U256::from(1000 - sfee))?
            .checked_div(U256::from(1000))?;

        let numerator = amount_in_less_fee.checked_mul(reserve_out)?;
        let denominator = reserve_in.checked_add(amount_in_less_fee)?;
        let amount_out = numerator.checked_div(denominator)?;
        // 5. Calculate price impact with decimal adjustment

        let new_reserve_in = reserve_in.checked_add(amount_in_less_fee)?;
        let new_reserve_out = reserve_out.checked_sub(amount_out)?;

        // Multiply numerator first to preserve precision (like fixed-point math)
        let scale = U256::from(10).pow(U256::from(18)); // or 1e6 if 1e18 feels too big
        let new_price = new_reserve_out
            .checked_mul(scale)?
            .checked_div(new_reserve_in)?;

        let (reserves0, reserves1) = {
            // Commit state
            if from0 {
                (new_reserve_in, new_reserve_out)
            } else {
                (new_reserve_out, new_reserve_in)
            }
        };

        Some(V2Trade {
            from0,
            amount_in,
            amount_out,
            fee_amount: amount_in.checked_sub(amount_in_less_fee)?,
            new_reserves0: reserves0,
            new_reserves1: reserves1,
            new_price,
        })
    }
}
#[derive(Debug, Default)]
pub struct V2Trade {
    pub fee_amount: U256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub from0: bool,
    pub new_price: U256,
    pub new_reserves0: U256,
    pub new_reserves1: U256,
}
