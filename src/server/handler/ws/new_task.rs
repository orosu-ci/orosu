use crate::model::api::ws::TaskMessage;
use crate::server::ServerState;
use crate::tasks::{TaskEvent, TaskLaunchResult};
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct NewTaskWebSocketHandler;

impl NewTaskWebSocketHandler {
    pub async fn handle_task_run_output(mut socket: WebSocket, state: Arc<ServerState>) {
        let Some(task_message_result) = socket.recv().await else {
            tracing::info!("Client disconnected");
            return;
        };
        let Ok(task_message) = task_message_result else {
            tracing::error!("Cannot receive task message");
            return;
        };

        let task_message_payload = match task_message {
            Message::Text(text) => match serde_json::from_str::<TaskMessage>(&text) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::error!("Cannot deserialize task message from string: {:?}", e);
                    return;
                }
            },
            Message::Binary(bytes) => match serde_json::from_slice::<TaskMessage>(&bytes) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::error!("Cannot deserialize task message from bytes: {:?}", e);
                    return;
                }
            },
            _ => {
                tracing::error!("Unsupported message type: {:?}", task_message);
                return;
            }
        };

        tracing::info!("Received task message: {:?}", task_message_payload);

        let (mut sender, mut receiver) = socket.split();

        let tasks = &state.tasks;

        match task_message_payload {
            TaskMessage::NewTask {
                script_key,
                task_id,
                arguments,
            } => {
                todo!();
                // let moved_script_key: DatabaseUuid = script_key.clone().into();
                // let script_path = state
                //     .database_pool
                //     .get()
                //     .await
                //     .unwrap()
                //     .interact(move |conn| {
                //         use crate::schema::scripts::dsl::*;
                //         scripts
                //             .select(path)
                //             .filter(key.eq(&moved_script_key).and(deleted_on.is_null()))
                //             .first::<String>(conn)
                //             .optional()
                //             .unwrap()
                //     })
                //     .await
                //     .unwrap();
                //
                // let Some(script_path) = script_path else {
                //     tracing::error!("Cannot get script path");
                //     return;
                // };
                //
                // let script_path_buf = script_path.clone().into();
                //
                // let Ok(task) = tasks
                //     .get_or_start(script_key, task_id, script_path_buf, arguments)
                //     .await
                // else {
                //     tracing::error!("Cannot get or start task");
                //     return;
                // };
                // match task {
                //     TaskLaunchResult::Created(task) => {
                //         tracing::info!(
                //             "Starting task {} for script {}: {}",
                //             task_id,
                //             script_key,
                //             script_path,
                //         );
                //         let mut rx = task.read().await.events_tx.subscribe();
                //         for event in task.read().await.events.iter() {
                //             tracing::info!("Task event history: {:?}", event);
                //             if let Err(e) = sender
                //                 .send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                //                 .await
                //             {
                //                 tracing::error!("Cannot send history event: {:?}", e);
                //                 break;
                //             };
                //         }
                //         while let Ok(event) = rx.recv().await {
                //             tracing::info!("Task event real-time: {:?}", event);
                //             if let Err(e) = sender
                //                 .send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                //                 .await
                //             {
                //                 tracing::error!("Cannot send real-time event: {:?}", e);
                //                 break;
                //             };
                //             if let TaskEvent::Finished(code) = event.event {
                //                 tracing::info!("Task finished with code {}", code);
                //                 break;
                //             }
                //         }
                //     }
                //     TaskLaunchResult::Finished(task) => {
                //         tracing::info!(
                //             "Task {} for script {} was already finished: {}",
                //             task_id,
                //             script_key,
                //             script_path
                //         );
                //         for event in task.events {
                //             sender
                //                 .send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                //                 .await
                //                 .unwrap();
                //         }
                //     }
                // }
            }
        }

        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Utf8Bytes::from_static("Task finished"),
            })))
            .await
        {
            tracing::error!("Cannot send close message: {:?}", e);
        };

        tracing::info!("Send close message");

        let wait_for_close = async {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Close(_)) => {
                        tracing::info!("Client disconnected");
                        return;
                    }
                    Ok(msg) => {
                        tracing::info!("Received message: {:?}", msg);
                    }
                    Err(e) => {
                        tracing::error!("Cannot receive message: {:?}", e);
                        return;
                    }
                }
            }

            tracing::info!("Client disconnected");
        };

        if timeout(Duration::from_secs(3), wait_for_close)
            .await
            .is_err()
        {
            tracing::warn!("Client did not close connection in time");
        }
    }
}
