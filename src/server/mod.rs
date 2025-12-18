use crate::client::Client;
use crate::configuration::ListenConfiguration;
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;

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
    clients: Vec<Client>,
}

pub struct Server {
    listen: ListenConfiguration,
    state: Arc<ServerState>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct WebAppAuthClaims {
    pub username: UserName,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub enum AuthScope {
    Worker,
}

pub type UserName = String;

#[derive(Clone, Debug)]
pub struct WorkerAuthContext {
    pub client: Client,
}

#[derive(Clone, Debug)]
pub struct UserAuthContext {
    pub header: Header,
    pub claims: WebAppAuthClaims,
}

#[derive(Clone, Debug)]
pub enum AuthContext {
    Worker(WorkerAuthContext),
}
