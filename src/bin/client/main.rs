use crate::arguments::CliArguments;
use axum::http::Uri;
use clap::Parser;
use orosu::api::client::ApiClient;
use std::str::FromStr;
use uuid::Uuid;

mod arguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let arguments = CliArguments::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .init();

    let mut parts = Uri::from_str(&arguments.address)?.into_parts();
    if parts.scheme.is_none() {
        parts.scheme = Some("wss".try_into()?);
    }
    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".try_into()?);
    }
    let uri = Uri::from_parts(parts)?;

    let client = ApiClient::connect(uri, arguments.key).await?;

    let run_id = Uuid::new_v4();

    client
        .start_task(run_id, arguments.variables, arguments.script)
        .await?;

    Ok(())
}
