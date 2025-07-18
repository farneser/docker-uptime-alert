use std::env;
use std::path::Path;
use std::sync::Arc;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

pub struct Config {
    pub docker_socket: String,
    pub server_port: u16,
    pub bot_token: String,
    pub admin_chat_id: i32,
}

static INSTANCE: Lazy<RwLock<Option<Arc<Config>>>> = Lazy::new(|| RwLock::new(None));

impl Config {
    fn new() -> Self {
        let docker_socket = fetch_env_variable("DOCKER_SOCKET_PATH")
            .unwrap_or_else(|| "/var/run/docker.sock".to_string());

        let server_port = fetch_env_variable("SERVER_PORT")
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(3000);

        let bot_token = fetch_bot_token().unwrap_or_else(|err| {
            eprintln!("Error fetching bot token: {}", err);
            std::process::exit(1);
        });
        
        let admin_chat_id = fetch_env_variable("ADMIN_CHAT_ID")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0);

        Config {
            docker_socket,
            server_port,
            bot_token,
            admin_chat_id,
        }
    }

    pub async fn instance() -> Arc<Config> {
        let mut instance = INSTANCE.write().await;

        if instance.is_none() {
            *instance = Some(Arc::new(Config::new()));
        }

        instance.clone().unwrap()
    }
}

pub fn load_env() {
    fn load_log_level() {
        let default_log_level = "info";
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| default_log_level.to_string());

        env::set_var("RUST_LOG", log_level);
    }

    let dotenv_path = ".env";

    if Path::new(dotenv_path).exists() {
        dotenv().expect("Failed to read '.env' file");

        print!("Successfully loaded .env file\n");
    } else {
        eprint!("Failed to find .env file. Using system environment variables instead.\n");
    }

    load_log_level();
}
fn fetch_env_variable(var: &str) -> Option<String> {
    env::var(var).ok()
}

fn fetch_bot_token() -> Result<String, String> {
    let val = fetch_env_variable("TELEGRAM_BOT_TOKEN");

    match val {
        None => Err("environment variable 'BOT_TOKEN' is not set".to_owned()),
        Some(_) => Ok(val.unwrap())
    }
}