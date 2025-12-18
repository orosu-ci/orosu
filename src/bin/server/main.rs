mod arguments;

use crate::arguments::CliArguments;
use clap::Parser;
use orosu::configuration::Configuration;
use orosu::server;
use orosu::tasks::Tasks;
use server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let arguments = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .init();

    let configuration = Configuration::from_file(&arguments.config_file_path)?;

    let tasks = Tasks::new();

    let server = Server::new(configuration.listen, configuration.clients, tasks);

    server.serve().await?;

    Ok(())
}
