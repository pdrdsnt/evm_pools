pub mod any_pool;
pub mod err;
pub mod sol_types;
pub mod v3_base;
pub mod v3_pool;
pub mod v4_pool;
#[cfg(test)]
mod tests {
    //USDC BSC-USD pool
    const V3_USDC_USD: &str = "0x2C3c320D49019D4f9A92352e947c7e5AcFE47D68";
    //usdt - bnb
    const V3_USDT_BNB_ADDR: &str = "0x47a90A2d92A8367A91EfA1906bFc8c1E05bf10c4";
    //cake - BSC-USD v3
    const V3_CAKE_USD_ADDR: &str = "0xFe4fe5B4575c036aC6D5cCcFe13660020270e27A";

    const V4_ADDR: &str = "0xd13Dd3D6E93f276FAfc9Db9E6BB47C1180aeE0c4";

    const BNB_PROVIDER: &str = "https://binance.llamarpc.com";
    const BNB_PROVIDER_2: &str = "https://bsc.rpc.blxrbdn.com";
    const BNB_PROVIDER_3: &str = "https://bsc-mainnet.public.blastapi.io";
    const BNB_PROVIDER_4: &str = "https://bsc.drpc.org";
    use std::{str::FromStr, sync::Arc};

    use alloy::{
        primitives::{Address, U256},
        providers::Provider,
    };

    use crate::any_pool::AnyPool;

    use super::*;

    #[tokio::test]
    pub async fn test_v3_trade_simulation() {
        let urls = vec![
            BNB_PROVIDER,
            BNB_PROVIDER_2,
            BNB_PROVIDER_3,
            BNB_PROVIDER_4,
        ];
        let provider = Arc::new(crate::any_pool::generate_fallback_provider(urls));

        let usdc_usd_address: Address = V3_USDC_USD.parse().unwrap();
        let usdt_bnb_address: Address = V3_USDT_BNB_ADDR.parse().unwrap();
        let cake_usd_address: Address = V3_CAKE_USD_ADDR.parse().unwrap();

        let mut usdc_usd_pool = AnyPool::create_v3(provider.clone(), usdc_usd_address)
            .await
            .unwrap();

        let mut usdt_bnb_pool = AnyPool::create_v3(provider.clone(), usdt_bnb_address)
            .await
            .unwrap();

        let mut cake_usd_pool = AnyPool::create_v3(provider.clone(), cake_usd_address)
            .await
            .unwrap();
        let amount_in = U256::from(1000000000000_u64); // 1 USD
        let from_token_0 = true;

        let usdc_usd_result = usdc_usd_pool.trade(amount_in, from_token_0).await;

        println!("usdc usd trade: {:?}", usdc_usd_result);

        let cake_usd_result = cake_usd_pool.trade(amount_in, from_token_0).await;

        println!("usdc usd trade: {:?}", cake_usd_result);

        let usdt_bnb_result = usdt_bnb_pool.trade(amount_in, from_token_0).await;

        println!("usdc usd trade: {:?}", usdt_bnb_result);
    }
}
