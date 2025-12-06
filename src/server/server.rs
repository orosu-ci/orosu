use crate::model::api::ErrorCode;
use crate::server::handler::{
    AuthHandler, ScriptsHandler, TasksHandler, WebAppHandler, WorkersHandler,
};
use crate::server::{AuthScope, Configuration, Server, ServerState};
use crate::tasks::Tasks;
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::middleware;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum_client_ip::{ClientIp, ClientIpSource};
use deadpool_diesel::sqlite::Pool;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

impl Server {
    pub fn new(configuration: Configuration, database_pool: Pool, tasks: Tasks) -> Self {
        let jwt_decoding_key =
            DecodingKey::from_base64_secret(configuration.jwt_secret.as_str()).unwrap();
        let jwt_encoding_key =
            EncodingKey::from_base64_secret(configuration.jwt_secret.as_str()).unwrap();
        let validation = Validation::default();
        let state = Arc::new(ServerState {
            database_pool,
            admin_username: configuration.admin_username.clone(),
            admin_password: configuration.admin_password.clone(),
            jwt_decoding_key,
            jwt_encoding_key,
            validation,
            tasks,
        });
        Self {
            configuration,
            state,
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let router = self.build_router();

        let addr =
            SocketAddr::from_str(format!("0.0.0.0:{}", self.configuration.listen_port).as_str())?;
        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
        Ok(())
    }

    fn build_router(&self) -> axum::Router {
        let mut web_client_router = axum::Router::new()
            .route("/api/auth/status", get(AuthHandler::status))
            .route("/api/auth/login", post(AuthHandler::login))
            .route("/api/auth/logout", post(AuthHandler::logout))
            .route("/api/scripts", get(ScriptsHandler::list))
            .route("/api/workers", get(WorkersHandler::list))
            .route("/api/workers", post(WorkersHandler::create))
            .route("/api/workers/{worker_id}", delete(WorkersHandler::delete))
            .route("/api/tasks", get(TasksHandler::attach))
            .route("/api/tasks/active", get(TasksHandler::active_tasks))
            .route("/{*path}", get(WebAppHandler::get))
            .route("/", get(WebAppHandler::get))
            .with_state(self.state.clone())
            .layer(TraceLayer::new_for_http())
            .layer(AuthScope::WebApp.into_extension());
        let whitelisted_ips = &self.configuration.whitelisted_ips;
        if let Some(ips) = whitelisted_ips {
            web_client_router = web_client_router
                .layer(middleware::from_fn_with_state(ips.clone(), whitelist_layer))
                .layer(ClientIpSource::RightmostXForwardedFor.into_extension());
        }
        let worker_router = axum::Router::new()
            .route("/api/worker/tasks", get(TasksHandler::attach))
            .with_state(self.state.clone())
            .layer(TraceLayer::new_for_http())
            .layer(AuthScope::Worker.into_extension());

        worker_router.merge(web_client_router)
    }
}

async fn whitelist_layer(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    State(whitelist): State<Vec<IpAddr>>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();
    match ClientIp::from_request_parts(&mut parts, &()).await {
        Ok(ip) => {
            if !whitelist.contains(&ip.0) {
                return ErrorCode::Forbidden.into_response();
            }
        }
        Err(_) => {
            let ip = remote_addr.ip();
            if !whitelist.contains(&ip) {
                return ErrorCode::Forbidden.into_response();
            }
        }
    }
    next.run(Request::from_parts(parts, body)).await
}
