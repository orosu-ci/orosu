use crate::api::envelopes::{
    TaskEventResponseEnvelope, TaskLaunchRequestEnvelope, TaskLaunchStatusResponseEnvelope,
};
use crate::api::{ServerErrorResponse, ServerTaskNotification, StartTaskRequest, TaskLaunchStatus};
use crate::tasks::TaskOutput;
use anyhow::Context;
use axum::http::Uri;
use axum::http::header::AUTHORIZATION;
use futures_util::{SinkExt, StreamExt};
use std::process::exit;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

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
            format!("Bearer {key}")
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
        arguments: Vec<String>,
        script_name: String,
    ) -> anyhow::Result<()> {
        let mut ws_stream = self.ws_stream.lock().await;

        let start_task_request = TaskLaunchRequestEnvelope {
            body: StartTaskRequest {
                script_name,
                arguments,
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
            anyhow::bail!("Server did not respond with a valid response, got {response}")
        };

        let response: TaskLaunchStatusResponseEnvelope = response_bytes.into();

        match response {
            TaskLaunchStatusResponseEnvelope::Success { body, .. } => {
                match body {
                    TaskLaunchStatus::Launched { .. } => {
                        // do nothing
                    }
                    TaskLaunchStatus::Running { output, .. } => {
                        for line in output {
                            line.value.print();
                        }
                    }
                    TaskLaunchStatus::Finished {
                        output, exit_code, ..
                    } => {
                        for line in output {
                            line.value.print();
                        }
                        exit(exit_code);
                    }
                }
            }
            TaskLaunchStatusResponseEnvelope::Failure { error, .. } => error.panic(),
        };

        while let Some(event) = ws_stream.next().await {
            let event = event?;
            match event {
                Message::Binary(event) => {
                    let event: TaskEventResponseEnvelope = event.into();
                    match event {
                        TaskEventResponseEnvelope::Success { body, .. } => match body {
                            ServerTaskNotification::Output(output) => {
                                output.value.print();
                            }
                            ServerTaskNotification::ExitCode(exit_code) => {
                                ws_stream.send(Message::Close(None)).await?;
                                exit(exit_code);
                            }
                        },
                        TaskEventResponseEnvelope::Failure { error, .. } => {
                            ws_stream.send(Message::Close(None)).await?;
                            error.panic();
                        }
                    }
                }
                Message::Close(cause) => {
                    tracing::info!("WebSocket closed: {cause:?}");
                    break;
                }
                _ => {
                    panic!("Unexpected message: {event:?}");
                }
            }
        }

        Ok(())
    }
}

impl TaskOutput {
    fn print(&self) {
        match self {
            TaskOutput::Stdout(line) => println!("{line}"),
            TaskOutput::Stderr(line) => eprintln!("{line}"),
        }
    }
}

impl ServerErrorResponse {
    fn panic(&self) {
        match self {
            ServerErrorResponse::CannotLaunchScript => panic!("Cannot launch script"),
            ServerErrorResponse::ScriptNotFound => panic!("Script not found"),
            ServerErrorResponse::Unknown => panic!("Unknown error"),
        }
    }
}
