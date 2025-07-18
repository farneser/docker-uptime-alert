mod bot;
mod cfg;
mod monitor;
mod runnable;
mod server;
mod sender;

use crate::bot::TelegramBot;
use crate::monitor::Monitor;
use crate::runnable::Runnable;
use crate::server::Server;

use bollard::{Docker, API_DEFAULT_VERSION};
use log::error;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub struct AlertMessage {
    pub container_id: String,
    pub message: String,
    pub timestamp: Instant,
    pub chat_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ContainerStatus {
    pub is_healthy: bool,
    pub last_unhealthy: Option<Instant>,
    pub acknowledged: bool,
}

#[derive(Clone, Debug)]
pub struct AppContainer {
    pub docker: Docker,
    pub counter: Arc<AtomicU32>,
    pub container_states: Arc<Mutex<HashMap<String, ContainerStatus>>>,
    pub alert_queue: Arc<Mutex<VecDeque<AlertMessage>>>,
    pub teloxide_bot: Arc<teloxide::Bot>,
}

impl AppContainer {
    pub fn new(docker: Docker, bot: teloxide::Bot) -> Self {
        AppContainer {
            docker,
            counter: Arc::new(AtomicU32::new(0)),
            container_states: Arc::new(Mutex::new(HashMap::new())),
            alert_queue: Arc::new(Mutex::new(VecDeque::new())),
            teloxide_bot: Arc::new(bot),
        }
    }

    pub async fn acknowledge(&self, container_id: &str) {
        let mut states = self.container_states.lock().await;
        if let Some(state) = states.get_mut(container_id) {
            state.acknowledged = true;
        }
    }
}

#[tokio::main]
async fn main() {
    cfg::load_env();
    pretty_env_logger::init();

    let config = cfg::Config::instance().await;

    let docker = Docker::connect_with_socket(config.docker_socket.as_str(), 20, API_DEFAULT_VERSION)
            .expect("Failed to connect to Docker");

    let bot = teloxide::Bot::new(config.bot_token.to_string());

    let container = Arc::new(AppContainer::new(docker, bot));

    let threads: Vec<Box<dyn Runnable<AppContainer> + Send + Sync>> = vec![
        Box::new(Monitor::new(Arc::clone(&container))),
        Box::new(TelegramBot::new(Arc::clone(&container))),
        Box::new(Server::new(config.server_port, Arc::clone(&container))),
        Box::new(sender::Sender::new(Arc::clone(&container))),
    ];

    let handles = threads
        .into_iter()
        .map(|t| {
            tokio::spawn(async move {
                t.run().await;
            })
        })
        .collect::<Vec<_>>();

    tokio::select! {
        _ = async {
            for handle in handles {
                if let Err(e) = handle.await {
                    eprintln!("Task failed: {:?}", e);
                }
            }
        } => {
            error!("All tasks completed");
        },
        _ = signal::ctrl_c() => {
            error!("Received Ctrl+C. Shutting down...");
        },
    }
}
