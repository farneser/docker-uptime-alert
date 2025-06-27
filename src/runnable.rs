use std::sync::Arc;

#[async_trait::async_trait]
pub trait Runnable<C>: Send + Sync
where
    C: Send + Sync,
{
    fn get_container(&self) -> Arc<C>;

    async fn run(&self);
}
