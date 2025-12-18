use crate::client::Client;
use crate::configuration::ListenConfiguration;
use std::sync::Arc;

mod auth_scope;
mod handler;
mod server;

pub struct ServerState {
    clients: Vec<Client>,
}

pub struct Server {
    listen: ListenConfiguration,
    state: Arc<ServerState>,
}

#[derive(Clone, Debug)]
pub enum AuthScope {
    Worker,
}

#[derive(Clone, Debug)]
pub struct WorkerAuthContext {
    pub client: Client,
}

#[derive(Clone, Debug)]
pub enum AuthContext {
    Worker(WorkerAuthContext),
}
