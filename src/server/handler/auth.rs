use crate::model::api::{
    AuthSignInRequestPayload, AuthSignInResponsePayload, AuthSignOutResponsePayload,
    AuthStatusResponsePayload, ErrorCode, ResponsePayload,
};
use crate::server::handler::AuthHandler;
use crate::server::{ServerState, UserAuthContext, WebAppAuthClaims};
use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{encode, Header};
use std::ops::Add;
use std::sync::Arc;

impl AuthHandler {
    pub async fn status(
        user_auth: Result<UserAuthContext, ErrorCode>,
    ) -> ResponsePayload<AuthStatusResponsePayload> {
        let is_authenticated = match user_auth {
            Ok(_) => true,
            Err(_) => false,
        };
        ResponsePayload::Success(AuthStatusResponsePayload { is_authenticated })
    }

    pub async fn login(
        State(state): State<Arc<ServerState>>,
        Json(payload): Json<AuthSignInRequestPayload>,
    ) -> Result<(CookieJar, ResponsePayload<AuthSignInResponsePayload>), ErrorCode> {
        if payload.username != state.admin_username || payload.password != state.admin_password {
            return Err(ErrorCode::Unauthorized);
        }

        let claims = WebAppAuthClaims {
            username: payload.username,
            exp: chrono::Utc::now()
                .add(chrono::Duration::hours(1))
                .timestamp() as usize,
        };

        let Ok(token) = encode(&Header::default(), &claims, &state.jwt_encoding_key) else {
            return Err(ErrorCode::Unknown);
        };

        let cookie = Cookie::build(("nerdy_releaser_token", token))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .path("/")
            .build();
        let cookie_jar = CookieJar::new().add(cookie);

        Ok((
            cookie_jar,
            ResponsePayload::Success(AuthSignInResponsePayload { success: true }),
        ))
    }

    pub async fn logout(
        _: UserAuthContext,
    ) -> (CookieJar, ResponsePayload<AuthSignOutResponsePayload>) {
        let cookie = Cookie::build(("nerdy_releaser_token", ""))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .path("/")
            .build();
        let cookie_jar = CookieJar::new().add(cookie);

        (
            cookie_jar,
            ResponsePayload::Success(AuthSignOutResponsePayload { success: true }),
        )
    }
}
