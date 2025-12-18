use crate::api::UserAgentHeader;
use crate::server::{AuthContext, AuthScope, ServerState, WorkerAuthContext};
use axum::Extension;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::http::header::USER_AGENT;

impl AuthScope {
    pub const fn into_extension(self) -> Extension<Self> {
        Extension(self)
    }
}

impl FromRequestParts<Arc<ServerState>> for AuthContext {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> Result<Self, Self::Rejection> {
        let scope = parts
            .extensions
            .get::<AuthScope>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let Some(user_agent_header) = parts.headers.get(USER_AGENT) else {
            tracing::error!("User agent header is missing");
            return Err(StatusCode::UNAUTHORIZED);
        };

        let user_agent_header: UserAgentHeader = match user_agent_header.try_into() {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("Unexpected user agent header format: {e}");
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        match scope {
            AuthScope::Worker => {
                let Some(auth_header) = &parts.headers.get(AUTHORIZATION) else {
                    tracing::error!("Authorization header is missing");
                    return Err(StatusCode::UNAUTHORIZED);
                };
                let auth_header_value =
                    auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
                let parts = auth_header_value
                    .split_once(' ')
                    .ok_or(StatusCode::UNAUTHORIZED)?;
                if parts.0 != "Bearer" {
                    tracing::error!("Invalid authorization header format");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                let token = parts.1;

                let Some(client) = state.clients.iter().find(|e| e.secret == token) else {
                    tracing::error!("Client with provided secret not found");
                    return Err(StatusCode::UNAUTHORIZED);
                };

                tracing::info!(
                    "Client {}, version {} authenticated",
                    client.name,
                    user_agent_header.version
                );

                Ok(AuthContext::Worker(WorkerAuthContext {
                    client: client.clone(),
                }))
            }
        }
    }
}

impl FromRequestParts<Arc<ServerState>> for WorkerAuthContext {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> Result<Self, Self::Rejection> {
        let context = AuthContext::from_request_parts(parts, state).await?;
        match context {
            AuthContext::Worker(token) => Ok(token),
        }
    }
}
