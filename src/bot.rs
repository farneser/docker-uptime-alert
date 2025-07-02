use crate::runnable::Runnable;
use crate::AppContainer;
use log::info;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use teloxide::respond;
use teloxide::types::Message;

#[derive(Clone, Debug)]
pub struct TelegramBot {
    container: Arc<AppContainer>,
}

impl TelegramBot {
    pub fn new(app_container: Arc<AppContainer>) -> Self {
        Self {
            container: app_container,
        }
    }

    pub fn get_teloxide_bot(&self) -> Arc<teloxide::Bot> {
        Arc::clone(&self.container.teloxide_bot)
    }
}

#[async_trait::async_trait]
impl Runnable<AppContainer> for TelegramBot {
    fn get_container(&self) -> Arc<AppContainer> {
        Arc::clone(&self.container)
    }

    async fn run(&self) {
        info!("Starting Telegram bot...");

        let bot = self.get_teloxide_bot();
        let container = Arc::clone(&self.container);

        teloxide::repl(bot, move |msg: Message| {
            let container = Arc::clone(&container);

            async move {
                if let Some(text) = msg.text() {
                    info!("Received message: {}", text);

                    container.counter.fetch_add(1, Ordering::SeqCst);
                } else {
                    info!("Received a non-text message");
                }
                respond(())
            }
        })
        .await;
    }
}
