use crate::arguments::CliArguments;
use anyhow::Context;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use clap::Parser;
use ed25519_dalek::ed25519::signature::rand_core::OsRng;
use ed25519_dalek::SigningKey;
use orosu::client_key::ClientKey;
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

    let key = SigningKey::generate(&mut OsRng);

    let private_key = key.to_bytes();
    let public_key = key.verifying_key().to_bytes();

    let client_key = ClientKey {
        client_name: name,
        key: private_key.into(),
    };

    let private_key = rkyv::to_bytes::<rkyv::rancor::Error>(&client_key)
        .context("failed to serialize client key")?;

    let private_key = STANDARD.encode(private_key);
    let public_key = STANDARD.encode(public_key);

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
