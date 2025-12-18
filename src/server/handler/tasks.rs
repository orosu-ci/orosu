use crate::api::envelopes::{
    RequestEnvelope, TaskEventResponseEnvelope, TaskLaunchStatusResponseEnvelope,
};
use crate::api::{ServerErrorResponse, ServerTaskNotification, StartTaskRequest, TaskLaunchStatus};
use crate::client::Client;
use crate::server::AuthContext;
use crate::server::handler::TasksHandler;
use crate::tasks::TaskLaunchResult;
use crate::tasks::task::Task;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, FromRequestParts, Request, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

impl TasksHandler {
    pub async fn attach(
        ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
        auth_context: AuthContext,
        ws: WebSocketUpgrade,
        request: Request,
    ) -> impl IntoResponse {
        let client = match auth_context {
            AuthContext::Worker(worker_auth_context) => worker_auth_context.client,
        };

        let (mut parts, _) = request.into_parts();

        let ip = ClientIp::from_request_parts(&mut parts, &())
            .await
            .map(|e| e.0)
            .unwrap_or_else(|_| remote_addr.ip());

        if let Some(whitelist) = &client.whitelisted_ips {
            if !whitelist.iter().any(|cidr| cidr.contains(&ip)) {
                tracing::warn!("Client {} is not whitelisted for {}", ip, client.name);
                return StatusCode::FORBIDDEN.into_response();
            }
        };

        if let Some(blacklist) = &client.blacklisted_ips {
            if blacklist.iter().any(|cidr| cidr.contains(&ip)) {
                tracing::warn!("Client {} is blacklisted for {}", ip, client.name);
                return StatusCode::FORBIDDEN.into_response();
            }
        }

        ws.on_upgrade(move |socket| handle_task_run_output(socket, client))
    }
}

async fn handle_task_run_output(mut socket: WebSocket, client: Client) {
    let Some(task_message_result) = socket.recv().await else {
        tracing::info!("Client disconnected");
        _ = socket.send(Message::Close(None)).await;
        return;
    };
    let Ok(task_message) = task_message_result else {
        tracing::error!("Cannot receive task message");
        _ = socket.send(Message::Close(None)).await;
        return;
    };

    let Message::Binary(start_task_message) = task_message else {
        tracing::error!("Cannot deserialize task message");
        _ = socket.send(Message::Close(None)).await;
        return;
    };

    let Ok(start_task_message_payload) =
        serde_json::from_slice::<RequestEnvelope<StartTaskRequest>>(&start_task_message)
    else {
        tracing::error!("Cannot deserialize task message from bytes");
        _ = socket.send(Message::Close(None)).await;
        return;
    };

    tracing::info!("Received task message: {:?}", start_task_message_payload);

    let (mut sender, mut receiver) = socket.split();

    let arguments = start_task_message_payload.body.arguments;

    let script_name = start_task_message_payload.body.script_name;
    let script = client.scripts.iter().find(|e| e.name == script_name);

    let Some(script) = script else {
        tracing::error!("Script {} not found", script_name);
        let error_message = TaskLaunchStatusResponseEnvelope::Failure {
            error: ServerErrorResponse::ScriptNotFound,
        };
        _ = sender.send(Message::Binary(error_message.into())).await;
        _ = sender.send(Message::Close(None)).await;
        return;
    };

    let task = Task::create(script.clone());

    let TaskLaunchResult {
        created_on,
        output,
        handler,
    } = match task.run(arguments).await {
        Ok(task) => task,
        Err(e) => {
            tracing::error!("Unable to launch script {}: {:?}", script_name, e);
            let error_message = TaskLaunchStatusResponseEnvelope::Failure {
                error: ServerErrorResponse::CannotLaunchScript,
            };
            _ = sender.send(Message::Binary(error_message.into())).await;
            _ = sender.send(Message::Close(None)).await;
            return;
        }
    };

    let created_message = TaskLaunchStatusResponseEnvelope::Success {
        body: TaskLaunchStatus::Launched {
            started_on: created_on,
        },
    };
    _ = sender.send(Message::Binary(created_message.into())).await;

    tracing::info!("Starting task for script {}", script_name);

    let mut rx = output.subscribe();

    let mut handler_fuse = handler;
    let exit_code = loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                match maybe_event {
                    Ok(event) => {
                        tracing::info!("Task event: {:?}", event);
                        let message = TaskEventResponseEnvelope::Success {
                            body: ServerTaskNotification::Output(event),
                        };
                        if let Err(e) = sender.send(Message::Binary(message.into())).await {
                            tracing::error!("Cannot send real-time event: {:?}", e);
                            break None;
                        };
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                        tracing::warn!("Receiver lagged by {} messages", count);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::warn!("Receiver was closed");
                    }
                }
            }
            res = &mut handler_fuse => break Some(res.unwrap()),
        }
    };
    let Some(exit_code) = exit_code else {
        tracing::info!("Script {} was not awaited to be finished", script_name);
        return;
    };
    tracing::info!(
        "Script {} has finished with {} exit code",
        script_name,
        exit_code
    );
    let message = TaskEventResponseEnvelope::Success {
        body: ServerTaskNotification::ExitCode(exit_code),
    };
    if let Err(e) = sender.send(Message::Binary(message.into())).await {
        tracing::error!("Cannot send exit-code event: {:?}", e);
    };

    if let Err(e) = sender.send(Message::Close(None)).await {
        tracing::error!("Cannot send close message: {:?}", e);
    };

    tracing::info!("Send close message");

    let wait_for_close = async {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(cause)) => {
                    tracing::info!("Client disconnected: {:?}", cause);
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
