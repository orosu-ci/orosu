use crate::model::api::ErrorCode;
use crate::server::{AuthContext, AuthScope, ServerState, UserAuthContext, WorkerAuthContext};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::Extension;
use std::sync::Arc;

impl AuthScope {
    pub const fn into_extension(self) -> Extension<Self> {
        Extension(self)
    }
}

impl FromRequestParts<Arc<ServerState>> for AuthContext {
    type Rejection = ErrorCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> Result<Self, Self::Rejection> {
        let scope = parts
            .extensions
            .get::<AuthScope>()
            .cloned()
            .ok_or(ErrorCode::Unauthorized)?;

        match scope {
            AuthScope::Worker => {
                let Some(auth_header) = &parts.headers.get(AUTHORIZATION) else {
                    tracing::error!("Authorization header is missing");
                    return Err(ErrorCode::Unauthorized);
                };
                let auth_header_value =
                    auth_header.to_str().map_err(|_| ErrorCode::Unauthorized)?;
                let parts = auth_header_value
                    .split_once(' ')
                    .ok_or(ErrorCode::Unauthorized)?;
                if parts.0 != "Bearer" {
                    tracing::error!("Invalid authorization header format");
                    return Err(ErrorCode::Unauthorized);
                }
                let token = parts.1;

                let Some(client) = state.clients.iter().find(|e| e.secret == token) else {
                    tracing::error!("Client with provided secret not found");
                    return Err(ErrorCode::Unauthorized);
                };

                Ok(AuthContext::Worker(WorkerAuthContext {
                    client: client.clone(),
                }))
            }
        }
    }
}

impl FromRequestParts<Arc<ServerState>> for UserAuthContext {
    type Rejection = ErrorCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> Result<Self, Self::Rejection> {
        let context = AuthContext::from_request_parts(parts, state).await?;
        match context {
            AuthContext::Worker(_) => Err(ErrorCode::Unauthorized),
        }
    }
}

impl FromRequestParts<Arc<ServerState>> for WorkerAuthContext {
    type Rejection = ErrorCode;

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
