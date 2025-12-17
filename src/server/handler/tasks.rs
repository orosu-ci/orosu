use crate::model::api::{ErrorCode, ResponsePayload, TaskResponsePayload};
use crate::server::handler::ws::new_task::NewTaskWebSocketHandler;
use crate::server::handler::TasksHandler;
use crate::server::{AuthContext, ServerState};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;

impl TasksHandler {
    pub async fn attach(
        auth_context: AuthContext,
        State(server_state): State<Arc<ServerState>>,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        match auth_context {
            AuthContext::Worker(worker_auth_context) => {
                tracing::info!("Worker {} logged in", worker_auth_context.worker_id)
            }
        }

        ws.on_upgrade(move |socket| {
            NewTaskWebSocketHandler::handle_task_run_output(socket, server_state)
        })
    }
}
