use orosu::configuration::LogLevelConfiguration;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    pub variables: Vec<String>,
    #[clap(short, long)]
    pub address: String,
    #[clap(short, long)]
    pub script: String,
    #[clap(short, long)]
    pub key: String,
    #[clap(short, long)]
    pub retries: Option<u8>,
    #[clap(short, long, default_value = "info")]
    pub log_level: LogLevelConfiguration,
    #[clap(short, long)]
    pub file: Option<Vec<String>>,
    #[clap(short, long, default_value_t = 1024)]
    pub chunk_size: usize,
}
