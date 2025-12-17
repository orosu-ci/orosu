use orosu::{scripts, server};

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Configuration {
    #[clap(flatten)]
    pub server_configuration: server::Configuration,
    #[clap(flatten)]
    pub scripts_configuration: scripts::Configuration,
}
