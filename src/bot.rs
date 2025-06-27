use crate::AppContainer;
use log::info;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use teloxide::respond;
use teloxide::types::Message;
use crate::runnable::Runnable;

#[derive(Clone, Debug)]
pub struct TelegramBot {
    teloxide_bot: Arc<teloxide::Bot>,
    container: Arc<AppContainer>,
}

impl TelegramBot {
    pub fn new(token: String, app_container: Arc<AppContainer>) -> Self {
        let bot = teloxide::Bot::new(token);

        TelegramBot {
            teloxide_bot: Arc::new(bot),
            container: app_container,
        }
    }

    pub fn get_teloxide_bot(&self) -> Arc<teloxide::Bot> {
        Arc::clone(&self.teloxide_bot)
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
