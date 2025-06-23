mod bot;
mod server;
mod cfg;

use crate::bot::TelegramBot;
use crate::server::{AppContainer, Server};
use bollard::{Docker, API_DEFAULT_VERSION};
use std::sync::Arc;
use tokio::signal;
use log::{error, info, warn};


#[tokio::main]
async fn main() {
    cfg::load_env();
    
    pretty_env_logger::init();

    let config = cfg::Config::instance().await;
    
    let docker = Docker::connect_with_socket(
        config.docker_socket.as_str(),
        20,
        API_DEFAULT_VERSION,
    )
    .expect("Failed to connect to Docker");

    let container = Arc::new(AppContainer::new(docker));
    let server = Server::new(config.server_port, Arc::clone(&container));
    let bot = TelegramBot::new(config.bot_token.to_string());

    tokio::select! {
        _ = server.run() => {
            error!("Web server exited");
        },
        _ = bot.run() => {
            error!("Telegram bot exited");
        },
        _ = signal::ctrl_c() => {
            error!("Received Ctrl+C. Shutting down...");
        },
    }
}
