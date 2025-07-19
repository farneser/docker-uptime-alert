use crate::runnable::Runnable;
use crate::AppContainer;
use std::sync::Arc;
use teloxide::requests::Requester;
use tokio::time::{self, Duration};

pub struct Sender {
    pub container: Arc<AppContainer>,
}

impl Sender {
    pub fn new(container: Arc<AppContainer>) -> Self {
        Self { container }
    }
}

#[async_trait::async_trait]
impl Runnable<AppContainer> for Sender {
    fn get_container(&self) -> Arc<AppContainer> {
        Arc::clone(&self.container)
    }

    async fn run(&self) {
        let container = self.get_container();

        loop {
            let mut messages = container.alert_queue.lock().await;

            while let Some(alert) = messages.pop_front() {
                let bot = container.teloxide_bot.clone();
                let message = format!(
                    "[{}] {}: {}",
                    alert.timestamp.elapsed().as_secs(),
                    alert.container_id,
                    alert.message
                );

                if let Err(e) = bot.send_message(alert.chat_id.unwrap(), message).await {
                    eprintln!("Failed to send message: {}", e);
                }
            }

            drop(messages);

            time::sleep(Duration::from_millis(100)).await;
        }
    }
}