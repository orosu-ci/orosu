use crate::model::api::TaskResponsePayload;
use crate::model::DatabaseTask;
use crate::server::ServerState;
use axum::extract::ws::{Message, WebSocket};
use deadpool_diesel::sqlite::Pool;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::join;
use tokio::task::JoinHandle;

pub struct ActiveTasksWebSocketHandler;

impl ActiveTasksWebSocketHandler {
    pub async fn handle_active_tasks(mut socket: WebSocket, state: Arc<ServerState>) {
        tracing::info!(
            "New websocket connection established for active tasks, sending initial message"
        );

        let tasks = match Self::list_active_tasks(state.database_pool.clone()).await {
            Ok(tasks) => tasks,
            Err(e) => {
                tracing::error!(
                    "Error listing active tasks: {}. Terminating ws connection",
                    e
                );
                _ = socket.send(Message::Close(None)).await;
                return;
            }
        };
        if let Err(e) = socket
            .send(Message::text(serde_json::to_string(&tasks).unwrap()))
            .await
        {
            tracing::error!("Error sending initial message: {}", e);
            return;
        };

        let (mut tx, mut rx) = socket.split();

        let mut stream = state.tasks.changes_tx.subscribe();
        loop {
            tokio::select! {
                client_message = rx.next() => {
                    match client_message {
                        None => {
                            tracing::info!("Websocket connection terminated");
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("Error receiving message: {}", e);
                            break;
                        }
                        Some(Ok(Message::Close(_))) => {
                            tracing::info!("Received close message, terminating websocket");
                            break;
                        }
                        Some(Ok(message)) => {
                            tracing::info!("Received message: {:?}", message);
                        }
                    }
                }
                task_update = stream.recv() => {
                    let Ok(message) = task_update else {
                        tracing::error!("Error receiving task update");
                        break;
                    };
                    tracing::info!("Tasks updated: {:?}", message);

                    let active_tasks = match Self::list_active_tasks(state.database_pool.clone()).await {
                        Ok(tasks) => tasks,
                        Err(e) => {
                            tracing::error!(
                                "Error listing active tasks: {}. Terminating ws connection",
                                e
                            );
                            _ = tx.send(Message::Close(None)).await;
                            return;
                        }
                    };

                    let active_tasks_count = active_tasks.len();
                    if let Err(e) = tx.send(Message::text(serde_json::to_string(&active_tasks).unwrap())).await {
                        tracing::error!("Error sending message: {}", e);
                        break;
                    };
                    tracing::info!("Sent {} active tasks", active_tasks_count);
                }
            }
        }

        tracing::info!("Websocket connection terminated");
    }

    async fn list_active_tasks(pool: Pool) -> anyhow::Result<Vec<TaskResponsePayload>> {
        pool.get()
            .await
            .unwrap()
            .interact(|conn| {
                use crate::schema::tasks::dsl::*;
                tasks
                    .select(DatabaseTask::as_select())
                    .filter(finished_on.is_null())
                    .load::<DatabaseTask>(conn)
                    .map(|result| result.into_iter().map(Into::into).collect())
            })
            .await
            .map_err(|e| anyhow::anyhow!("Interaction error: {}", e))?
            .map_err(Into::into)
    }
}
