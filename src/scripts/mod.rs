mod scripts_watcher;

use deadpool_diesel::sqlite::Pool;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Configuration {
    #[arg(
        long,
        env = "SCRIPTS_DIRECTORY",
        help = "Path to the scripts directory"
    )]
    pub scripts_directory: PathBuf,
}

pub struct ScriptsWatcher {
    directory: PathBuf,
    database: Pool,
}
