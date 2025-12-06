use crate::server::handler::ws::active_tasks::ActiveTasksWebSocketHandler;
use crate::server::handler::ws::new_task::NewTaskWebSocketHandler;
use crate::server::handler::TasksHandler;
use crate::server::{AuthContext, ServerState, UserAuthContext};
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
            AuthContext::WebApp(user_auth_context) => {
                tracing::info!("User {} logged in", user_auth_context.claims.username);
            }
            AuthContext::Worker(worker_auth_context) => {
                tracing::info!("Worker {} logged in", worker_auth_context.worker_id)
            }
        }

        ws.on_upgrade(move |socket| {
            NewTaskWebSocketHandler::handle_task_run_output(socket, server_state)
        })
    }

    pub async fn active_tasks(
        _: UserAuthContext,
        State(server_state): State<Arc<ServerState>>,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| {
            ActiveTasksWebSocketHandler::handle_active_tasks(socket, server_state)
        })
    }
}
