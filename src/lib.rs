mod any_pool;
mod err;
mod v3_base;
mod v3_pool;
mod v4_pool;

mod sol_types;
//USDC BSC-USD pool
const V3_USDC_USD: &str =
    "0x2C3c320D49019D4f9A92352e947c7e5AcFE47D68";
//usdt - bnb
const V3_USDT_BNB_ADDR: &str =
    "0x47a90A2d92A8367A91EfA1906bFc8c1E05bf10c4";
//cake - BSC-USD v3
const V3_CAKE_USD_ADDR: &str =
    "0xFe4fe5B4575c036aC6D5cCcFe13660020270e27A";

const V4_ADDR: &str =
    "0xd13Dd3D6E93f276FAfc9Db9E6BB47C1180aeE0c4";

const BNB_PROVIDER: &str = "https://binance.llamarpc.com";
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        primitives::{
            Address, U256,
            aliases::{I24, U24},
        },
        transports::http::reqwest::Url,
    };

    use crate::{
        any_pool::AnyPool,
        sol_types::PoolKey,
        v3_base::{tick_math, trade_math},
    };

    use super::*;
    #[tokio::test]
    pub async fn v3_0() {
        println!("testing for usdc bsc v3 pool");
        let mut any_pool = AnyPool::create_v3(
            Url::from_str(BNB_PROVIDER).unwrap(),
            Address::from_str(V3_USDC_USD).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V3(mut pool, key, contract)) =
            any_pool
        {
            println!(
                "v3 trade simulation: {:?}",
                trade_math::trade(
                    &pool,
                    &key.fee,
                    U256::ONE << 64,
                    true
                )
            );
        }
    }

    #[tokio::test]
    pub async fn v4() {
        let pool_key: PoolKey = PoolKey {
    currency0: Address::from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d").unwrap(),
    currency1: Address::from_str("0xAB3dBcD9B096C3fF76275038bf58eAC10D22C61f").unwrap(),
    fee: U24::from(100_u8).to(),
    tickSpacing: I24::ONE,
    hooks: Address::ZERO,
};

        let mut any_pool = AnyPool::create_v4(
            pool_key,
            Url::from_str(BNB_PROVIDER).unwrap(),
            Address::from_str(V4_ADDR).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V4(mut pool, key, contract)) =
            any_pool
        {
            println!(
                "v4 trade simulation: {:?}",
                trade_math::trade(
                    &pool,
                    &key.fee,
                    U256::ONE << 64,
                    true
                )
                .ok()
            );
        }
    }

    #[test]
    fn test_price_to_tick() {
        let range = 1..10;
        for n in range {
            println!(
                "testing price: {}",
                U256::from(4295128739u64) << n,
            );
            let _tick = tick_math::tick_from_price(
                U256::from(4295128739u64) << n,
            )
            .unwrap();
        }
    }
    #[test]
    fn test_tick_to_price() {
        let min_tick = -10;
        let max_tick = 10;

        let p_tick = I24::try_from(min_tick).unwrap();
        let mut prev_price =
            tick_math::price_from_tick(p_tick).unwrap();

        for tick in (min_tick + 1)..max_tick {
            let c_tick = I24::try_from(tick).unwrap();
            let cur_price =
                tick_math::price_from_tick(c_tick).unwrap();

            if cur_price < prev_price {
                println!(
                    "Non‑monotonic at tick {}: {} vs {}",
                    tick, prev_price, cur_price
                );
                prev_price = cur_price;
            }
        }
    }
}
