use std::{num::NonZeroUsize, str::FromStr};

use alloy::{
    rpc::client::RpcClient,
    transports::{http::Http, layers::FallbackLayer},
};
use alloy_provider::{Provider, ProviderBuilder};
use reqwest::Url;
use tower::ServiceBuilder;

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
    use std::fs::write;

    use alloy::{
        primitives::{Address, U256},
        rpc::types::{Bundle, Filter},
    };
    use alloy_sol_types::{SolCall, SolEvent};
    use futures::future::join_all;

    use crate::{
        any_pool::AnyPool,
        pool::UniPool,
        sol_types::{
            PoolKey,
            StateView::StateViewInstance,
        },
        v4_pool::V4Pool,
    };

    use super::*;

    #[tokio::test]
    pub async fn test() {
        let v4_key: PoolKey = PoolKey {
            currency0: Address::from_str("0x55d398326f99059fF775485246999027B3197955")
                .unwrap(),
            currency1: Address::from_str("0x8d0D000Ee44948FC98c9B98A4FA4921476f08B0d")
                .unwrap(),
            fee: alloy::primitives::aliases::U24::from(43),
            tickSpacing: alloy::primitives::aliases::I24::try_from(60_i32).unwrap(),
            hooks: Address::ZERO,
        };

        let bnb_provider_urls = vec![
            BNB_PROVIDER.to_string(),
            BNB_PROVIDER_2.to_string(),
            BNB_PROVIDER_3.to_string(),
            BNB_PROVIDER_4.to_string(),
        ];

        let provider = generate_fallback_provider(bnb_provider_urls);
        let current_block = provider.get_block_number().await.unwrap();
        let step = 500;
        let mut from_block = current_block;
        let mut to_block = current_block - step;
        let min_block = 50000000;
        println!("block {}", current_block);
        let mut keys = Vec::new();

        keys.push(v4_key);
        let mut keis = Vec::new();

        while to_block > min_block {
            let v4_created_event_filter = Filter::new()
                .address(Address::from_str(V4_ADDR).unwrap())
                .from_block(to_block)
                .to_block(from_block);

            let logs = provider.get_logs(&v4_created_event_filter).await;

            match logs {
                Ok(mut ok) => {
                    println!("{} - {}", from_block, to_block);
                    println!("all v4 pools {:?}", ok);
                    keis.append(&mut ok);
                    let mut data = String::new();
                    for k in ok {
                        data += &k.data().data.to_string();
                    }
                    if !data.is_empty() {
                        if let Err(er) = write(
                            format!("./v4_events/{}-{}.logs", from_block, to_block),
                            data,
                        ) {
                            println!("err saving {}", er);
                        }
                    }
                }
                Err(err) => println!("err getting logs {:?}", err),
            }

            from_block -= step;
            to_block -= step;
        }

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

        let amount_in = U256::from(1000000000000_u64); // 1 USD
        let from_token_0 = true;
        let mut pools = Vec::new();

        /*
                for p in v2_pools {
                    if let Some(pool) =
                        V2Pool::create_v2_from_address(p, Some(3000), provider.clone()).await
                    {
                        pools.push(AnyPool::V2(pool));
                    }
                }

                for p in v3_pools {
                    if let Ok(v3_pool) = V3Pool::new_from_address(p, provider.clone()).await {
                        pools.push(AnyPool::V3(v3_pool))
                    }
                }
        */

        let v4_state_view =
            StateViewInstance::new(V4_ADDR.parse().unwrap(), provider.clone());

        for key in keys.clone() {
            if let Ok(v4_pool) = V4Pool::new(key.into(), v4_state_view.clone()).await {
                pools.push(AnyPool::V4(v4_pool));
            }
        }

        let calls = {
            let mut _calls = Vec::new();
            for pool in &pools {
                _calls.push(Bundle {
                    transactions: pool.create_sync_call(),
                    block_override: None,
                });
            }
            _calls
        };

        match provider.call_many(calls.as_slice()).await {
            Ok(results) => {
                for (i, result) in results.into_iter().enumerate() {
                    println!("decoding result {:?}", result);
                    let current_pool = &mut pools[i];
                    println!("for pool {:?}", current_pool);

                    match current_pool {
                        AnyPool::V2(v2_pool) => {
                            match v2_pool.decode_sync_result(result) {
                                Ok(_) => (),
                                Err(err) => {
                                    println!("error updating v2 {:?}", err)
                                }
                            };
                        }
                        AnyPool::V3(v3_pool) => {
                            match v3_pool.decode_sync_result(result) {
                                Ok(_) => (),
                                Err(err) => eprintln!("error updating v3 {:?}", err),
                            }
                        }
                        AnyPool::V4(v4_pool) => {
                            match v4_pool.decode_sync_result(result) {
                                Ok(_) => (),
                                Err(err) => println!("error updating v4 {:?}", err),
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("batch call err: {}", err);
                println!("trying normal requests");
                let mut fut = Vec::new();
                for p in &mut pools {
                    let update = p.super_sync();
                    fut.push(update);
                }

                for result in join_all(fut).await {}

                for p in &pools {
                    match p {
                        AnyPool::V2(v2_pool) => {
                            println!("v2 pool {:?}", v2_pool.contract.address());
                            println!("token 0 {:?}", v2_pool.get_a());
                            println!("token 0 {:?}", v2_pool.get_b());
                            println!("price {:?}", v2_pool.get_price());
                            println!("liquidity {}", v2_pool.get_liquidity());
                        }
                        AnyPool::V3(v3_pool) => {
                            println!("v3 pool {:?}", v3_pool.contract.address());
                            println!("token 0 {:?}", v3_pool.get_a());
                            println!("token 1 {:?}", v3_pool.get_b());
                            println!("liquidity {:?}", v3_pool.get_liquidity());

                            println!("price {:?}", v3_pool.get_price());
                        }

                        AnyPool::V4(v4_pool) => {
                            println!("v4 pool {:?}", v4_pool.contract.address());
                            println!("token 0 {:?}", v4_pool.get_a());
                            println!("token 1 {:?}", v4_pool.get_b());
                            println!("liquidity {:?}", v4_pool.get_liquidity());

                            println!("price {:?}", v4_pool.get_price());
                        }
                    }
                }
            }
        }

        println!("done");
        /* */
    }
}

pub fn generate_fallback_provider(urls: Vec<String>) -> impl Provider + Clone {
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
