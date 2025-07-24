use std::str::FromStr;
mod any_pool;
mod conncentrated_liquidity_pools;
mod v3_pool;
mod v4_pool;
use crate::sol_types::{PoolKey, V3Pool};
use alloy::primitives::{Address, aliases::U24};
mod err;
mod generator;
mod sol_types;
//USDC BSC-USD pool
const v3_usdc_usd_addr: &str =
    "0x2C3c320D49019D4f9A92352e947c7e5AcFE47D68";
//usdt - bnb
const v3_usdt_bnb_addr: &str =
    "0x47a90A2d92A8367A91EfA1906bFc8c1E05bf10c4";
//cake - BSC-USD v3
const v3_cake_usd_addr: &str =
    "0xFe4fe5B4575c036aC6D5cCcFe13660020270e27A";

const v4_addr: &str =
    "0xd13Dd3D6E93f276FAfc9Db9E6BB47C1180aeE0c4";

const bnb_provider: &str = "https://binance.llamarpc.com";
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        primitives::{
            Address, U256, aliases::I24, map::HashMap,
        },
        transports::http::reqwest::Url,
    };
    use alloy_provider::ProviderBuilder;

    use crate::{
        sol_types::V3Pool::V3PoolInstance,
        v3_state::AnyPool,
    };

    use super::*;
    #[tokio::test]
    pub async fn v3_0() {
        println!("testing for usdc bsc v3 pool");
        let mut any_pool = generator::create_v3(
            Url::from_str(bnb_provider).unwrap(),
            Address::from_str(v3_usdc_usd_addr).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V3(mut pool, contract)) =
            any_pool
        {
            println!(
                "v3 trade simulation: {:?}",
                tick_math::trade(
                    &pool,
                    U256::ONE << 64,
                    true
                )
            );
        }
    }
    #[tokio::test]
    pub async fn v3_1() {
        println!("testing for usdt bnb pool");
        let mut any_pool = generator::create_v3(
            Url::from_str(bnb_provider).unwrap(),
            Address::from_str(v3_usdt_bnb_addr).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V3(mut pool, contract)) =
            any_pool
        {
            println!(
                "v3 trade simulation: {:?}",
                tick_math::trade(
                    &pool,
                    U256::ONE << 64,
                    true
                )
            );
        }
    }

    #[tokio::test]
    pub async fn v3_2() {
        println!("testing for cake usd v3 pool");
        let mut any_pool = generator::create_v3(
            Url::from_str(bnb_provider).unwrap(),
            Address::from_str(v3_cake_usd_addr).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V3(mut pool, contract)) =
            any_pool
        {
            println!(
                "v3 trade simulation: {:?}",
                tick_math::trade(
                    &pool,
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

        let mut any_pool = generator::create_v4(
            pool_key,
            Url::from_str(bnb_provider).unwrap(),
            Address::from_str(v4_addr).unwrap(),
        )
        .await;
        if let Ok(AnyPool::V4(mut pool, contract)) =
            any_pool
        {
            println!(
                "v4 trade simulation: {:?}",
                tick_math::trade(
                    &pool,
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
            let tick = tick_math::tick_from_price(
                U256::from(4295128739u64) << n,
            )
            .unwrap();
        }
    }
    #[test]
    fn test_tick_to_price() {
        let min_tick = -10;
        let max_tick = 10;

        let mut p_tick = I24::try_from(min_tick).unwrap();
        let mut prev_price =
            tick_math::price_from_tick(p_tick).unwrap();

        for tick in (min_tick + 1)..max_tick {
            let c_tick = I24::try_from(tick).unwrap();
            let mut cur_price =
                tick_math::price_from_tick(c_tick).unwrap();

            if (cur_price < prev_price) {
                println!(
                    "Non‑monotonic at tick {}: {} vs {}",
                    tick, prev_price, cur_price
                );
                prev_price = cur_price;
            }
        }
    }
}
