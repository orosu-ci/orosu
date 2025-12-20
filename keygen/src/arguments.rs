use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(long)]
    pub private_key_output: Option<PathBuf>,
    #[clap(long)]
    pub public_key_output: Option<PathBuf>,
}
