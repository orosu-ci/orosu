use nerdy_releaser_api::database;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Configuration {
    #[clap(flatten)]
    pub database_configuration: database::Configuration,
}
