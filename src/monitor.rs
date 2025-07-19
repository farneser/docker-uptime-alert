use crate::cfg::Config;
use crate::AppContainer;
use crate::{AlertMessage, ContainerStatus, Runnable};
use bollard::models::ContainerSummaryStateEnum;
use bollard::query_parameters::ListContainersOptionsBuilder;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{self, Instant as TokioInstant};

pub struct Monitor {
    pub container: Arc<AppContainer>,
}

impl Monitor {
    pub fn new(container: Arc<AppContainer>) -> Self {
        Self { container }
    }
    async fn check_containers(&self, container: &Arc<AppContainer>, chat_id: &str) {
        let options = Some(ListContainersOptionsBuilder::default().all(true).build());

        match container.docker.list_containers(options).await {
            Ok(containers) => {
                container.counter.store(containers.len() as u32, std::sync::atomic::Ordering::SeqCst);

                let mut states = container.container_states.lock().await;
                let mut alerts = container.alert_queue.lock().await;

                for info in containers {
                    let id = info.id.clone().unwrap_or_else(|| "unknown".into());
                    let name = info.names.clone().unwrap_or_default().get(0).cloned().unwrap_or_else(|| id.clone());

                    let is_healthy = matches!(info.state, Some(ContainerSummaryStateEnum::RUNNING));
                    let stopped = matches!(
                        info.state,
                        Some(ContainerSummaryStateEnum::EXITED)
                            | Some(ContainerSummaryStateEnum::DEAD)
                            | Some(ContainerSummaryStateEnum::REMOVING)
                            | Some(ContainerSummaryStateEnum::CREATED)
                            | Some(ContainerSummaryStateEnum::PAUSED)
                    );

                    let state = states.entry(id.clone()).or_insert(ContainerStatus {
                        is_healthy: true,
                        last_unhealthy: None,
                        acknowledged: false,
                    });

                    let now = Instant::now();

                    if !is_healthy {
                        if state.is_healthy {
                            state.is_healthy = false;
                            state.last_unhealthy = Some(now.into());

                            let msg = if stopped {
                                format!("Container {} has stopped.", name)
                            } else {
                                format!("Container {} just became unhealthy.", name)
                            };

                            alerts.push_back(AlertMessage {
                                container_id: id.clone(),
                                message: msg,
                                timestamp: now.into(),
                                chat_id: Some(chat_id.to_string()),
                            });
                        } else if let Some(since) = state.last_unhealthy {
                            if now.duration_since(since.into()) >= Duration::from_secs(1800) && !state.acknowledged {
                                alerts.push_back(AlertMessage {
                                    container_id: id.clone(),
                                    message: format!("Container {} is STILL unhealthy after 30 minutes.", name),
                                    timestamp: now.into(),
                                    chat_id: Some(chat_id.to_string()),
                                });

                                state.last_unhealthy = Some(now.into());
                            }
                        }
                    } else if !state.is_healthy {
                        alerts.push_back(AlertMessage {
                            container_id: id.clone(),
                            message: format!("Container {} has recovered and is now healthy.", name),
                            timestamp: now.into(),
                            chat_id: Some(chat_id.to_string()),
                        });

                        state.is_healthy = true;
                        state.last_unhealthy = None;
                        state.acknowledged = false;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error listing containers: {}", e);
            }
        }
    }
}

#[async_trait::async_trait]
impl Runnable<AppContainer> for Monitor {
    fn get_container(&self) -> Arc<AppContainer> {
        Arc::clone(&self.container)
    }

    async fn run(&self) {
        let container = self.get_container();
        let chat_id = Config::instance().await.admin_chat_id.to_string();

        self.check_containers(&container, &chat_id).await;

        let start = TokioInstant::now();
        let mut interval = time::interval_at(
            start + Duration::from_secs(60 - start.elapsed().as_secs() % 60),
            Duration::from_secs(60),
        );

        loop {
            interval.tick().await;
            self.check_containers(&container, &chat_id).await;
        }
    }
}