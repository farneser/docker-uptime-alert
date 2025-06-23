use bollard::{Docker};

use axum::response::{Html};
use axum::routing::get;
use axum::Router;
use bollard::query_parameters::ListContainersOptionsBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::State;
use log::info;
use tokio::net;

#[derive(Clone)]
pub struct AppContainer {
    pub docker: Docker,
}

impl AppContainer {
    pub fn new(docker: Docker) -> Self {
        AppContainer { docker }
    }
}

pub struct Server {
    port: u16,
    app_container: Arc<AppContainer>
}

impl Server {
    pub fn new(port: u16, app_container: Arc<AppContainer>) -> Self {
        Server { port, app_container }
    }

    pub async fn run(self) {

        let router = Router::new()
            .route("/", get(docker_status))
            .with_state(self.app_container);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        let listener = net::TcpListener::bind(&addr).await.unwrap();

        info!("Listening on {}", addr);

        axum::serve(listener, router).await.unwrap();
    }
}


async fn docker_status(
    State(container): State<Arc<AppContainer>>,
) -> Html<String> {

    let options = Some(ListContainersOptionsBuilder::default().all(true).build());

    let containers = container.docker.list_containers(options).await;

    match containers {
        Ok(containers) => {
            let mut response = String::new();
            let count = containers.iter().count().to_string();

            response.push_str(format!("<h1>Docker Containers Count: {}</h1>", count).as_str());

            response.push_str("<table border='1'>");

            for container in containers {
                let id = container.id.unwrap_or_else(|| "N/A".to_string());
                let image = container.image.unwrap_or_else(|| "N/A".to_string());
                let status = container.status.unwrap_or_else(|| "N/A".to_string());

                response.push_str("<tr>");

                response.push_str(&format!("<td>{}</td>", id));
                response.push_str(&format!("<td>{}</td>", image));
                response.push_str(&format!("<td>{}</td>", status));
                response.push_str(&format!(
                    "<td><a href='https://hub.docker.com/r/{}'>Docker Hub</a></td>",
                    image
                ));

                response.push_str("</tr>");
            }

            response.push_str("</table>");

            if response.is_empty() {
                response = "No containers found.".to_string();
            }

            Html(response)
        }
        Err(e) => {
            eprintln!("Error listing containers: {}", e);
            Html("Error listing containers".to_string())
        }
    }
}
