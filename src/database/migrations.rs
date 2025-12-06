use crate::database::Configuration;
use anyhow::Context;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_pending_migrations(configuration: &Configuration) -> anyhow::Result<Option<usize>> {
    let mut connection = SqliteConnection::establish(&configuration.database_file)
        .context("Cannot establish a database connection")?;

    let result = connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Cannot apply migrations")?;

    match result.len() {
        0 => Ok(None),
        count => Ok(Some(count)),
    }
}
