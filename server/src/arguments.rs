use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    #[cfg_attr(
        target_os = "linux",
        clap(
            short,
            long,
            env = "CONFIG_FILE",
            default_value = "/etc/orosu/config.yaml"
        )
    )]
    #[cfg_attr(not(target_os = "linux"), clap(short, long, env = "CONFIG_FILE"))]
    pub config_file_path: PathBuf,
}
