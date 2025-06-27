use crate::runnable::Runnable;
use crate::AppContainer;
use axum::extract::State;
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

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        let listener = net::TcpListener::bind(&addr).await.unwrap();

        info!("Listening on {}", addr);

        axum::serve(listener, router).await.unwrap();
    }
}

use std::fmt::Write;

async fn docker_status(State(container): State<Arc<AppContainer>>) -> Html<String> {
    let mut messages = container.alert_queue.lock().await;
    let mut alerts_html = String::new();

    if !messages.is_empty() {
        alerts_html.push_str("<h2>Alert Messages</h2><ul>");
        while let Some(alert) = messages.pop_front() {
            write!(
                alerts_html,
                "<li>[{}] {}: {}</li>",
                alert.timestamp.elapsed().as_secs(),
                alert.container_id,
                alert.message
            ).unwrap();
        }
        alerts_html.push_str("</ul>");
    } else {
        alerts_html.push_str("<h2>No alerts at the moment.</h2>");
    }

    Html(alerts_html)
}