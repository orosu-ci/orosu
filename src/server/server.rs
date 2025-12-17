use crate::model::api::ErrorCode;
use crate::server::handler::TasksHandler;
use crate::server::{AuthScope, Configuration, Server, ServerState};
use crate::tasks::Tasks;
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_client_ip::ClientIp;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

impl Server {
    pub fn new(configuration: Configuration, tasks: Tasks) -> Self {
        let jwt_decoding_key =
            DecodingKey::from_base64_secret(configuration.jwt_secret.as_str()).unwrap();
        let jwt_encoding_key =
            EncodingKey::from_base64_secret(configuration.jwt_secret.as_str()).unwrap();
        let validation = Validation::default();
        let state = Arc::new(ServerState {
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
        axum::Router::new()
            .route("/api/worker/tasks/manage", get(TasksHandler::attach))
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
