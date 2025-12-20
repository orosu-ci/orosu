use crate::arguments::CliArguments;
use anyhow::Context;
use axum::http::Uri;
use clap::Parser;
use orosu::api::client::ApiClient;

mod arguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    _ = dotenvy::dotenv();

    let arguments = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .init();

    let mut parts = Uri::try_from(&arguments.address)
        .context("invalid server address format")?
        .into_parts();
    if parts.scheme.is_none() {
        parts.scheme = Some("wss".try_into()?);
    }
    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".try_into()?);
    }
    let uri = Uri::from_parts(parts)?;

    let client = ApiClient::connect(uri, arguments.key)
        .await
        .context("failed to connect to server")?;

    client
        .start_task(arguments.variables, arguments.script)
        .await?;

    Ok(())
}
