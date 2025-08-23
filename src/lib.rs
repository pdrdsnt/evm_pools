use std::{num::NonZeroUsize, str::FromStr};

use alloy::{
    rpc::client::RpcClient,
    transports::{http::Http, layers::FallbackLayer},
};
use alloy_provider::{Provider, ProviderBuilder};
use reqwest::Url;
use tower::ServiceBuilder;

pub mod any_factory;
pub mod any_pool;
pub mod any_trade;
pub mod err;
pub mod pool_contract;

pub mod sol_types;
pub mod v2_pool;
pub mod v3_base;

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
    use std::sync::Arc;

    use alloy::primitives::{Address, U256};

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
        let provider = Arc::new(generate_fallback_provider(urls));

        let amount_in = U256::from(1000000000000_u64); // 1 USD
        let from_token_0 = true;

        let usdc_usd_address: Address = V3_USDC_USD.parse().unwrap();
        let usdt_bnb_address: Address = V3_USDT_BNB_ADDR.parse().unwrap();
        let cake_usd_address: Address = V3_CAKE_USD_ADDR.parse().unwrap();

        if let Some(mut usdc_usd_pool) =
            AnyPool::create_v3_from_address(usdc_usd_address, provider.clone()).await
        {
            println!("usdc usd pool created: {:?}", usdc_usd_pool);

            let usdc_usd_result = usdc_usd_pool.trade(amount_in, from_token_0);
            println!("usdc usd trade: {:?}", usdc_usd_result);
        }

        if let Some(mut usdt_bnb_pool) =
            AnyPool::create_v3_from_address(usdt_bnb_address, provider.clone()).await
        {
            println!("usdc usd pool created: {:?}", usdt_bnb_pool);

            let usdt_bnb_result = usdt_bnb_pool.trade(amount_in, from_token_0);
            println!("usdt bnb trade: {:?}", usdt_bnb_result);
        }

        if let Some(mut cake_usd_pool) =
            AnyPool::create_v3_from_address(cake_usd_address, provider.clone()).await
        {
            println!("usdc usd pool created: {:?}", cake_usd_pool);

            let cake_usd_result = cake_usd_pool.trade(amount_in, from_token_0);
            println!("cake usd trade: {:?}", cake_usd_result);
        }
    }
}
pub fn generate_fallback_provider(urls: Vec<&str>) -> impl Provider + Clone {
    let layer = FallbackLayer::default()
        .with_active_transport_count(NonZeroUsize::new(urls.len()).unwrap());
    let mut transports = Vec::new();
    for s in urls {
        if let Ok(url) = Url::from_str(&s) {
            transports.push(Http::new(url));
        }
    }

    let service = ServiceBuilder::new()
        .layer(layer)
        .service(transports);

    let cl = RpcClient::builder().transport(service, false);
    ProviderBuilder::new().connect_client(cl)
}
