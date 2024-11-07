use anyhow::Result;
use clap::Parser;

mod args;
mod client;

use args::{Args, Commands, GetCommands};
use client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        profile,
        region,
        verbose,
        command,
    } = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(match verbose {
            0 => tracing::Level::ERROR,
            1 => tracing::Level::WARN,
            2 => tracing::Level::INFO,
            3 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .init();

    let client = Client::new(profile, region).await?;

    match command {
        Commands::Get(cmd) => match cmd {
            GetCommands::Buckets => client.list_buckets().await?,
            GetCommands::Status { bucket, key } => client.get_status(&bucket, &key).await?,
            GetCommands::Report {
                bucket,
                key,
                output,
            } => client.get_report(&bucket, &key, output).await?,
            GetCommands::Transcript {
                bucket,
                key,
                output,
            } => client.get_transcript(&bucket, &key, output).await?,
        },
        Commands::Process(args) => client.process_file(args).await?,
    }

    Ok(())
}
