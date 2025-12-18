use crate::client::Client;
use crate::configuration::ListenConfiguration;
use crate::model::api::ErrorCode;
use crate::server::handler::TasksHandler;
use crate::server::{AuthScope, Server, ServerState};
use anyhow::Context;
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_client_ip::ClientIp;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

impl Server {
    pub fn new(listen: ListenConfiguration, clients: Vec<Client>) -> Self {
        let state = Arc::new(ServerState { clients });
        Self { listen, state }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let router = self.build_router();

        match &self.listen {
            ListenConfiguration::Tcp(address) => {
                let listener = tokio::net::TcpListener::bind(address)
                    .await
                    .with_context(|| "Failed to bind to TCP address")?;
                axum::serve(
                    listener,
                    router.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .await?;
            }

            ListenConfiguration::Socket(path) => {
                let listener = tokio::net::UnixListener::bind(path).with_context(|| {
                    format!("Failed to bind to unix socket path {}", path.display())
                })?;
                axum::serve(listener, router).await?;
            }
        };
        Ok(())
    }

    fn build_router(&self) -> axum::Router {
        axum::Router::new()
            .route("/", get(TasksHandler::attach))
            .with_state(self.state.clone())
            .layer(TraceLayer::new_for_http())
            .layer(AuthScope::Worker.into_extension())
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
