use std::sync::Arc;
use log::info;
use teloxide::respond;
use teloxide::types::Message;

#[derive(Clone, Debug)]
pub struct TelegramBot {
    teloxide_bot: Arc<teloxide::Bot>,
}

impl TelegramBot {
    pub async fn run(&self) {
        
        info!("Starting Telegram bot...");
        
        let bot = self.teloxide_bot.clone();

        teloxide::repl(bot, |msg: Message| async move {
            if let Some(text) = msg.text() {
                info!("Received message: {}", text);
            } else {
                info!("Received a non-text message");
            }
            respond(())
        })
        .await;
    }
}

impl TelegramBot {
    pub fn new(token: String) -> Self {
        let bot = teloxide::Bot::new(token);

        TelegramBot {
            teloxide_bot: Arc::new(bot),
        }
    }

    pub fn get_teloxide_bot(&self) -> Arc<teloxide::Bot> {
        self.teloxide_bot.clone()
    }
}
