use crate::arguments::CliArguments;
use anyhow::Context;
use axum::http::Uri;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use clap::Parser;
use orosu::api::client::ApiClient;
use orosu::client_key::ClientKey;
use tracing::level_filters::LevelFilter;

mod arguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    _ = dotenvy::dotenv();

    let arguments = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::from_level(arguments.log_level.into()))
        .compact()
        .init();

    let key = STANDARD
        .decode(arguments.key)
        .context("invalid key format")?;
    let key = rkyv::from_bytes::<ClientKey, rkyv::rancor::Error>(&key)
        .context("invalid key format")?
        .into();

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

    let client = ApiClient::connect(uri, key)
        .await
        .context("failed to connect to server")?;

    client
        .start_task(arguments.variables, arguments.script)
        .await?;

    Ok(())
}
