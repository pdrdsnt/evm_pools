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

pub mod pool;
pub mod sol_types;
pub mod v2_base;
pub mod v2_pool;
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

    const V2_BABYDODGE_USD: &str = "0xc736ca3d9b1e90af4230bd8f9626528b3d4e0ee0";

    const V2_CAKE_WBNB: &str = "0x0ed7e52944161450477ee417de9cd3a859b14fd0";

    const BNB_PROVIDER: &str = "https://binance.llamarpc.com";
    const BNB_PROVIDER_2: &str = "https://bsc.rpc.blxrbdn.com";
    const BNB_PROVIDER_3: &str = "https://bsc-mainnet.public.blastapi.io";
    const BNB_PROVIDER_4: &str = "https://bsc.drpc.org";
    use std::{fmt::Debug, sync::Arc};

    use alloy::primitives::{Address, U256};

    use crate::{
        any_pool::AnyPool, sol_types::PoolKey, v2_pool::V2Pool, v3_pool::V3Pool,
    };

    use super::*;

    #[tokio::test]
    pub async fn test() {
        let v4_key: PoolKey = PoolKey {
            currency0: Address::from_str("0x55d398326f99059fF775485246999027B3197955")
                .unwrap(),
            currency1: Address::from_str("0x55d398326f99059fF775485246999027B3197955")
                .unwrap(),
            fee: alloy::primitives::aliases::U24::from(6),
            tickSpacing: alloy::primitives::aliases::I24::try_from(60_i32).unwrap(),
            hooks: Address::ZERO,
        };

        let usdc_usd_address: Address = V3_USDC_USD.parse().unwrap();
        let usdt_bnb_address: Address = V3_USDT_BNB_ADDR.parse().unwrap();
        let cake_usd_address: Address = V3_CAKE_USD_ADDR.parse().unwrap();
        let v4_state_view: Address = V4_ADDR.parse().unwrap();

        let v2_babydodge_usd: Address = V2_BABYDODGE_USD.parse().unwrap();
        let v2_cake_wbnb: Address = V2_CAKE_WBNB.parse().unwrap();

        let v2_pools = vec![
            v2_babydodge_usd,
            v2_cake_wbnb,
        ];

        let v4_pools = vec![v4_state_view];
        let v3_pools = vec![
            usdt_bnb_address,
            cake_usd_address,
            usdc_usd_address,
        ];

        let bnb_provider_urls = vec![
            BNB_PROVIDER,
            BNB_PROVIDER_2,
            BNB_PROVIDER_3,
            BNB_PROVIDER_4,
        ];

        let provider = generate_fallback_provider(bnb_provider_urls);
        let amount_in = U256::from(1000000000000_u64); // 1 USD
        let from_token_0 = true;
        let mut pools = Vec::new();
        for p in v2_pools {
            if let Some(pool) =
                V2Pool::create_v2_from_address(p, Some(3000), provider.clone()).await
            {
                pools.push(AnyPool::V2(pool));
            }
        }
        for p in v3_pools {
            pools.push(AnyPool::V3(V3Pool::new(p, provider.clone())));
        }
        for p in v4_pools {
            pools.push(
                V2Pool::create_v2_from_address(p, Some(3000), provider.clone()).into(),
            );
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
