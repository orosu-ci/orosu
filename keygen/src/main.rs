use crate::arguments::CliArguments;
use clap::Parser;
use orosu::cryptography::Keygen;
use std::io;
use std::io::Write;

mod arguments;

fn prompt_input(prompt: &str) -> anyhow::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() -> anyhow::Result<()> {
    let arguments = CliArguments::parse();

    let name = match arguments.name {
        Some(name) => name,
        None => prompt_input("Name: ")?,
    };

    let keygen = Keygen::new(name);

    let private_key = keygen.private_key_base64();
    let public_key = keygen.public_key_base64();

    match arguments.private_key_output {
        Some(path) => {
            std::fs::write(&path, private_key)?;
            println!("Private key written to {}", path.display());
        }
        None => {
            println!("Private key: {private_key}");
        }
    };

    match arguments.public_key_output {
        Some(path) => {
            std::fs::write(&path, public_key)?;
            println!("Public key written to {}", path.display());
        }
        None => {
            println!("Public key: {public_key}");
        }
    };

    Ok(())
}
