use nerdy_releaser_api::{database, scripts, server};

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Configuration {
    #[clap(flatten)]
    pub database_configuration: database::Configuration,
    #[clap(flatten)]
    pub server_configuration: server::Configuration,
    #[clap(flatten)]
    pub scripts_configuration: scripts::Configuration,
}
