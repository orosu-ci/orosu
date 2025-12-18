use crate::api::envelopes::{
    TaskEventResponseEnvelope, TaskLaunchRequestEnvelope, TaskLaunchStatusResponseEnvelope,
};
use crate::api::{ServerTaskNotification, StartTaskRequest};
use anyhow::Context;
use axum::http::header::AUTHORIZATION;
use axum::http::Uri;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::event;
use uuid::Uuid;

pub struct ApiClient {
    ws_stream: Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl ApiClient {
    pub async fn connect(endpoint: Uri, key: String) -> anyhow::Result<Self> {
        let mut request = endpoint
            .clone()
            .into_client_request()
            .context("Cannot create request")?;
        request.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {}", key)
                .parse()
                .context("Invalid header value")?,
        );

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .context("Cannot connect")?;
        let ws_stream = Mutex::new(ws_stream);
        Ok(Self { ws_stream })
    }

    pub async fn start_task(
        &self,
        run_id: Uuid,
        arguments: Vec<String>,
        script_name: String,
    ) -> anyhow::Result<()> {
        let mut ws_stream = self.ws_stream.lock().await;

        let start_task_request = TaskLaunchRequestEnvelope {
            id: Uuid::new_v4(),
            body: StartTaskRequest {
                script_name,
                arguments,
                run_id,
            },
        };
        ws_stream
            .send(Message::Binary(start_task_request.into()))
            .await?;

        let Some(response) = ws_stream.next().await else {
            anyhow::bail!("Server did not respond")
        };
        let response = response?;

        let Message::Binary(response_bytes) = response else {
            anyhow::bail!(
                "Server did not respond with a valid response, got {}",
                response
            )
        };

        let response: TaskLaunchStatusResponseEnvelope = response_bytes.into();

        tracing::info!("Task launch response: {:?}", response);

        while let Some(event) = ws_stream.next().await {
            let event = event?;
            match event {
                Message::Binary(event) => {
                    let event: TaskEventResponseEnvelope = event.into();
                    match event {
                        TaskEventResponseEnvelope::Success { body, .. } => match body {
                            ServerTaskNotification::Output(output) => {
                                tracing::info!("Task output: {:?}", output);
                            }
                            ServerTaskNotification::ExitCode(exit_code) => {
                                tracing::info!("Task exited with code {}", exit_code);
                                ws_stream.send(Message::Close(None)).await?;
                            }
                        },
                        TaskEventResponseEnvelope::Failure { error, .. } => {
                            tracing::error!("Task failed: {:?}", error);
                            ws_stream.send(Message::Close(None)).await?;
                        }
                    }
                }
                Message::Close(cause) => {
                    tracing::info!("WebSocket closed: {:?}", cause);
                    break;
                }
                _ => {
                    event!(tracing::Level::ERROR, "Unexpected message: {:?}", event);
                }
            }
        }

        Ok(())
    }
}
