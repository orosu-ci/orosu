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
}
