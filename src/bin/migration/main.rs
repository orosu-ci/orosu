mod configuration;

use crate::configuration::Configuration;
use clap::Parser;
use nerdy_releaser_api::database::migrations::run_pending_migrations;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .init();

    let configuration = Configuration::parse();

    let migrations_applied = run_pending_migrations(&configuration.database_configuration)?;

    match migrations_applied {
        None => tracing::info!("No migrations to apply"),
        Some(count) => tracing::info!("Successfully applied {} migrations", count),
    }

    Ok(())
}
