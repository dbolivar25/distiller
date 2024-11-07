use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author = "Daniel Bolivar",
    version,
    about = "Distiller CLI: Summarize and Analyze Meeting Audio"
)]
pub(crate) struct Args {
    /// Override the AWS profile in your environment.
    #[arg(short, long, global = true, help_heading = "GLOBAL OPTIONS")]
    pub(crate) profile: Option<String>,

    /// Override the AWS region in your environment.
    #[arg(short, long, global = true, help_heading = "GLOBAL OPTIONS")]
    pub(crate) region: Option<String>,

    /// Increase the verbosity of the output. Warning: this can affect performance.
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        global = true,
        help_heading = "GLOBAL OPTIONS"
    )]
    pub(crate) verbose: u8,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Get resources and information from AWS.
    #[command(subcommand)]
    Get(GetCommands),
    /// Run the pipeline on a meeting audio file.
    Process(ProcessArgs),
}

#[derive(Subcommand)]
pub(crate) enum GetCommands {
    /// List the available S3 buckets.
    Buckets,
    /// Get the status of a transcript job for an audio file.
    Status {
        /// The S3 bucket in which the audio file was stored.
        bucket: String,
        /// The object key for the audio file.
        key: String,
    },
    /// Get the final report for an audio file.
    Report {
        /// The S3 bucket in which the audio file was stored.
        bucket: String,
        /// The object key for the audio file.
        key: String,
        /// Where to optionally save the report.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Get the transcript for an audio file.
    Transcript {
        /// The S3 bucket in which the audio file was stored.
        bucket: String,
        /// The object key for the audio file.
        key: String,
        /// Where to optionally save the transcript.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Parser)]
pub(crate) struct ProcessArgs {
    /// The S3 bucket in which to store the audio file.
    pub(crate) bucket: String,
    /// The object key for the audio file.
    pub(crate) file: PathBuf,
    /// The language of the audio file.
    #[arg(long)]
    pub(crate) language: Option<String>,
    /// Await the completion of the pipeline run.
    #[arg(long)]
    pub(crate) wait: bool,
    /// Where to optionally save the transcript.
    #[arg(long)]
    pub(crate) transcript_output: Option<PathBuf>,
    /// Where to optionally save the report.
    #[arg(long)]
    pub(crate) report_output: Option<PathBuf>,
}
