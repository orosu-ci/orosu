mod configuration;

use crate::configuration::Configuration;
use clap::Parser;
use orosu::server;
use orosu::tasks::Tasks;
use server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .init();

    let configuration = Configuration::parse();

    let server_configuration = configuration.server_configuration;
    let tasks = Tasks::new();

    let server = Server::new(server_configuration, tasks);

    server.serve().await?;

    Ok(())
}
