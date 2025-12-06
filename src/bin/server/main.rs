mod configuration;

use crate::configuration::Configuration;
use clap::Parser;
use deadpool_diesel::sqlite::{HookError, Manager, Pool};
use deadpool_diesel::{InteractError, Runtime};
use diesel::connection::SimpleConnection;
use nerdy_releaser_api::database::migrations::run_pending_migrations;
use nerdy_releaser_api::server;
use nerdy_releaser_api::tasks::Tasks;
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
    let database_configuration = configuration.database_configuration;

    let migrations_applied = run_pending_migrations(&database_configuration)?;

    match migrations_applied {
        None => { /* do nothing */ }
        Some(count) => tracing::info!("Successfully applied {} migrations", count),
    }

    let manager = Manager::new(database_configuration.database_file, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .post_create(deadpool_diesel::sqlite::Hook::async_fn(|obj, _| {
            Box::pin(async move {
                obj.interact(|conn| {
                    conn.batch_execute(
                        "
                        PRAGMA journal_mode = WAL;
                        PRAGMA busy_timeout = 5000;
                        PRAGMA synchronous = NORMAL;
                        ",
                    )
                })
                .await
                .map_err(|e| {
                    tracing::error!("Error interacting with database connection: {}", e);
                    HookError::message(format!("Failed to interact with SQLite connection: {}", e))
                })?
                .map_err(|e| {
                    tracing::error!("Error setting up database connection: {}", e);
                    HookError::message(format!("Failed to configure SQLite connection: {}", e))
                })
            })
        }))
        .build()?;
    let tasks = Tasks::new(pool.clone());

    let watcher = nerdy_releaser_api::scripts::ScriptsWatcher::new(
        configuration.scripts_configuration,
        pool.clone(),
    );

    tokio::spawn(async move { watcher.watch().await });

    let server = Server::new(server_configuration, pool, tasks);

    server.serve().await?;

    Ok(())
}
