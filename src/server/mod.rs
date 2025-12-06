use crate::tasks::Tasks;
use deadpool_diesel::sqlite::Pool;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

mod auth_scope;
mod handler;
mod server;

#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Configuration {
    #[arg(
        long,
        env = "LISTEN_PORT",
        default_value_t = 8080,
        help = "Port on which the dashboard will listen"
    )]
    pub listen_port: u16,
    #[arg(
        long,
        env = "WHITELISTED_IPS",
        help = "Comma separated list of IPs allowed to access the dashboard"
    )]
    pub whitelisted_ips: Option<Vec<IpAddr>>,
    #[arg(long, env = "JWT_SECRET", help = "JWT secret")]
    pub jwt_secret: String,
    #[arg(long, env = "ADMIN_USERNAME", help = "Admin username")]
    pub admin_username: String,
    #[arg(long, env = "ADMIN_PASSWORD", help = "Admin password")]
    pub admin_password: String,
}

pub struct ServerState {
    admin_username: String,
    admin_password: String,
    database_pool: Pool,
    jwt_decoding_key: DecodingKey,
    jwt_encoding_key: EncodingKey,
    validation: Validation,
    tasks: Tasks,
}

pub struct Server {
    configuration: Configuration,
    state: Arc<ServerState>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct WebAppAuthClaims {
    pub username: UserName,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub enum AuthScope {
    WebApp,
    Worker,
}

pub type UserName = String;

#[derive(Clone, Debug)]
pub struct WorkerAuthContext {
    pub worker_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct UserAuthContext {
    pub header: Header,
    pub claims: WebAppAuthClaims,
}

#[derive(Clone, Debug)]
pub enum AuthContext {
    WebApp(UserAuthContext),
    Worker(WorkerAuthContext),
}
