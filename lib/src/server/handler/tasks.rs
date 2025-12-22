use crate::api::envelopes::{
    RequestEnvelope, TaskEventResponseEnvelope, TaskLaunchStatusResponseEnvelope,
};
use crate::api::file_chunk::FileChunk;
use crate::api::{
    FileAttachment, ServerErrorResponse, ServerTaskNotification, StartTaskRequest, TaskLaunchStatus,
};
use crate::client::Client;
use crate::server::AuthContext;
use crate::server::handler::TasksHandler;
use crate::tasks::TaskLaunchResult;
use crate::tasks::task::Task;
use axum::Error;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, FromRequestParts, Request, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use futures_util::{SinkExt, StreamExt};
use md5::Digest;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::time::Duration;
use tempfile::{NamedTempFile, TempDir};
use tokio::time::timeout;
use zip::ZipArchive;

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

        if let Some(whitelist) = &client.whitelisted_ips
            && !whitelist.iter().any(|cidr| cidr.contains(&ip))
        {
            tracing::warn!("Client {} is not whitelisted for {}", ip, client.name);
            return StatusCode::FORBIDDEN.into_response();
        };

        if let Some(blacklist) = &client.blacklisted_ips
            && blacklist.iter().any(|cidr| cidr.contains(&ip))
        {
            tracing::warn!("Client {} is blacklisted for {}", ip, client.name);
            return StatusCode::FORBIDDEN.into_response();
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
    let attachment = start_task_message_payload.body.file;
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

    let attachment = match attachment {
        None => None,
        Some(attachment) => {
            let mut output = NamedTempFile::with_suffix(".zip").unwrap();
            let mut offset = 0;
            let size = attachment.size;
            let hash = attachment.hash;
            let mut hasher = md5::Context::new();
            while offset < size {
                let chunk_message = TaskLaunchStatusResponseEnvelope::Success {
                    body: TaskLaunchStatus::AwaitingFiles { offset },
                };
                _ = sender.send(Message::Binary(chunk_message.into())).await;
                let Some(response) = receiver.next().await else {
                    tracing::error!("Client disconnected during file transfer");
                    _ = sender.send(Message::Close(None)).await;
                    return;
                };
                let Ok(response) = response else {
                    tracing::error!("Cannot deserialize file chunk message");
                    _ = sender.send(Message::Close(None)).await;
                    return;
                };
                let Message::Binary(chunk) = response else {
                    tracing::error!("Cannot deserialize file chunk message");
                    _ = sender.send(Message::Close(None)).await;
                    return;
                };
                let Ok(chunk) = serde_json::from_slice::<RequestEnvelope<FileChunk>>(&chunk) else {
                    tracing::error!("Cannot deserialize file chunk message from bytes");
                    _ = sender.send(Message::Close(None)).await;
                    return;
                };
                let body = chunk.body;
                let chunk_offset = body.offset;
                if chunk_offset != offset {
                    tracing::error!("Unexpected chunk offset {chunk_offset}, expected {offset}");
                    return;
                }
                output.write_all(&body.data).unwrap();
                hasher.consume(&body.data);
                offset += body.data.len();
                tracing::debug!("Received attachment chunk with offset {chunk_offset}");
            }
            output.seek(SeekFrom::Start(0)).unwrap();
            tracing::debug!(
                "Finished attached file, saved into {}",
                output.path().display()
            );

            let computed_hash = hasher.finalize().0.to_vec();
            if computed_hash != hash {
                tracing::error!("File hash mismatch");
                let error_message = TaskLaunchStatusResponseEnvelope::Failure {
                    error: ServerErrorResponse::CannotLaunchScript,
                };
                _ = sender.send(Message::Binary(error_message.into())).await;
                _ = sender.send(Message::Close(None)).await;
                return;
            }
            tracing::debug!("File hash validated successfully");

            Some(output.into_file())
        }
    };

    let directory = match attachment {
        None => None,
        Some(mut file) => {
            let directory = TempDir::new().unwrap();
            tracing::debug!(
                "Created temporary directory for attached files: {}",
                directory.path().display()
            );

            let mut archive = ZipArchive::new(&mut file).unwrap();
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).unwrap();
                let outpath = directory.path().join(entry.name());

                if entry.is_dir() {
                    std::fs::create_dir_all(&outpath).unwrap();
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }
                    let mut outfile = File::create(&outpath).unwrap();
                    std::io::copy(&mut entry, &mut outfile).unwrap();
                }
                tracing::debug!("Extracted: {}", outpath.display());
            }
            tracing::debug!(
                "Successfully extracted archive to {}",
                directory.path().display()
            );

            Some(directory)
        }
    };

    let task = Task::create(script.clone());

    let TaskLaunchResult {
        created_on,
        output,
        handler,
    } = match task.run(arguments, directory).await {
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

    tracing::debug!("Send close message");

    let wait_for_close = async {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(cause)) => {
                    tracing::info!("Client disconnected: {:?}", cause);
                    return;
                }
                Ok(msg) => {
                    tracing::debug!("Received message: {:?}", msg);
                }
                Err(e) => {
                    tracing::error!("Cannot receive message: {:?}", e);
                    return;
                }
            }
        }

        tracing::debug!("Client disconnected");
    };

    if timeout(Duration::from_secs(3), wait_for_close)
        .await
        .is_err()
    {
        tracing::warn!("Client did not close connection in time");
    }
}
