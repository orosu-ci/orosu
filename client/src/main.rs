use crate::arguments::CliArguments;
use anyhow::Context;
use clap::Parser;
use orosu::api::client::ApiClient;
use orosu::cryptography::ClientKey;
use orosu::server_address::ServerAddress;
use tracing::level_filters::LevelFilter;

mod arguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::from_level(arguments.log_level.into()))
        .compact()
        .init();

    let key = ClientKey::from_string(arguments.key)?;
    let address = ServerAddress::from_string(arguments.address)?;

    let client = ApiClient::connect(address, key)
        .await
        .context("failed to connect to server")?;

    client
        .start_task(arguments.variables, arguments.script)
        .await?;

    Ok(())
}
