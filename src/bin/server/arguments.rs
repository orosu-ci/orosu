use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    #[clap(
        short,
        long,
        env = "CONFIG_FILE",
        default_value = "/etc/orosu/config.yaml"
    )]
    pub config_file_path: PathBuf,
}
