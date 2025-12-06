pub mod migrations;

#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Configuration {
    #[arg(long, env = "DATABASE_FILE", help = "Path to the database file")]
    pub database_file: String,
}
