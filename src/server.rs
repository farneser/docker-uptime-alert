use crate::runnable::Runnable;
use crate::AppContainer;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net;

pub struct Server {
    port: u16,
    app_container: Arc<AppContainer>,
}

impl Server {
    pub fn new(port: u16, app_container: Arc<AppContainer>) -> Self {
        Self { port, app_container }
    }
}

#[async_trait::async_trait]
impl Runnable<AppContainer> for Server {
    fn get_container(&self) -> Arc<AppContainer> {
        Arc::clone(&self.app_container)
    }

    async fn run(&self) {
        let container = self.get_container();

        let router = Router::new()
            .route("/", get(docker_status))
            .with_state(container);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        let listener = net::TcpListener::bind(&addr).await.unwrap();

        info!("Listening on {}", addr);

        axum::serve(listener, router).await.unwrap();
    }
}

async fn docker_status() -> Html<String> {
    let mut alerts_html = String::new();

    alerts_html.push_str("<h2>Working...</h2>");

    Html(alerts_html)
}