use bollard::models::ContainerSummaryStateEnum;
use std::sync::Arc;
use std::time::{Duration, Instant};
use bollard::query_parameters::ListContainersOptionsBuilder;
use crate::AppContainer;
use crate::{AlertMessage, ContainerStatus, Runnable};

pub struct Monitor {
    pub container: Arc<AppContainer>,
}

impl Monitor {
    pub fn new(container: Arc<AppContainer>) -> Self {
        Self { container }
    }
}

#[async_trait::async_trait]
impl Runnable<AppContainer> for Monitor {
    fn get_container(&self) -> Arc<AppContainer> {
        Arc::clone(&self.container)
    }

    async fn run(&self) {
        loop {
            let container = self.get_container();

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
                                });
                            } else if let Some(since) = state.last_unhealthy {
                                if now.duration_since(since.into()) >= Duration::from_secs(1800) && !state.acknowledged {
                                    alerts.push_back(AlertMessage {
                                        container_id: id.clone(),
                                        message: format!("Container {} is STILL unhealthy after 30 minutes.", name),
                                        timestamp: now.into(),
                                    });

                                    state.last_unhealthy = Some(now.into());
                                }
                            }
                        }

                        else if !state.is_healthy {
                            alerts.push_back(AlertMessage {
                                container_id: id.clone(),
                                message: format!("Container {} has recovered and is now healthy.", name),
                                timestamp: now.into(),
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

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
