use alloy_provider::Provider;

use crate::any_pool::AnyPool;

pub trait Factory<P: Provider + Clone> {
    // v2 needs ṕair, v3 needs pair and fee, v4 needs id. It is better to let the factories manage
    // internally
    async fn get_pools(&self) -> Vec<AnyPool<P>>;
}
