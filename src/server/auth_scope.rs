use crate::model::api::ErrorCode;
use crate::model::{DatabaseUuid, WorkerSecretKey};
use crate::server::{AuthContext, AuthScope, ServerState, UserAuthContext, WorkerAuthContext};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::Extension;
use axum_extra::extract::CookieJar;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use jsonwebtoken::decode;
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
            AuthScope::WebApp => {
                let cookies = CookieJar::from_request_parts(parts, state)
                    .await
                    .map_err(|_| ErrorCode::Unauthorized)?;
                let cookie = cookies
                    .get("nerdy_releaser_token")
                    .ok_or(ErrorCode::Unauthorized)?;

                let cookie_value = cookie.value();

                let token = decode(
                    &cookie_value.to_string(),
                    &state.jwt_decoding_key,
                    &state.validation,
                )
                .inspect_err(|e| tracing::error!("Cannot decode JWT token: {:?}", e))
                .map_err(|_| ErrorCode::Unauthorized)?;
                Ok(AuthContext::WebApp(UserAuthContext {
                    header: token.header,
                    claims: token.claims,
                }))
            }
            AuthScope::Worker => {
                let auth_header = &parts.headers[AUTHORIZATION];
                let auth_header_value =
                    auth_header.to_str().map_err(|_| ErrorCode::Unauthorized)?;
                let parts = auth_header_value
                    .split_once(' ')
                    .ok_or(ErrorCode::Unauthorized)?;
                if parts.0 != "Bearer" {
                    return Err(ErrorCode::Unauthorized);
                }
                let token = WorkerSecretKey::try_from(parts.1)
                    .inspect_err(|e| {
                        tracing::error!("Cannot parse worker secret key: {:?}", e);
                    })
                    .map_err(|_| ErrorCode::Unauthorized)?;

                let connection = state.database_pool.get().await.map_err(|e| {
                    tracing::error!("Cannot get database connection: {:?}", e);
                    ErrorCode::Unknown
                })?;

                let exists = connection
                    .interact(move |conn| {
                        use crate::schema::workers::dsl::*;
                        workers
                            .filter(secret_key.eq(token))
                            .select(id)
                            .first::<DatabaseUuid>(conn)
                            .optional()
                            .inspect_err(|e| {
                                tracing::error!("Cannot check if worker exists: {:?}", e)
                            })
                            .map_err(|_| ErrorCode::Unknown)
                    })
                    .await
                    .map_err(|_| ErrorCode::Unknown)?
                    .map_err(|_| ErrorCode::Unknown)?;
                let Some(id) = exists else {
                    return Err(ErrorCode::Unauthorized);
                };
                Ok(AuthContext::Worker(WorkerAuthContext {
                    worker_id: id.into(),
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
            AuthContext::WebApp(context) => Ok(context),
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
            AuthContext::WebApp(_) => Err(ErrorCode::Unauthorized),
            AuthContext::Worker(token) => Ok(token),
        }
    }
}
