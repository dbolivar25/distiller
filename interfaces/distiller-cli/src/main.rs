use anyhow::Result;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::DateTimeFormat;
use aws_sdk_sfn::{types::ExecutionStatus, Client as SfnClient};
use clap::{Parser, Subcommand};
use std::{path::PathBuf, time::Duration};
use tokio::{fs, time::sleep};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AWS profile to use
    #[arg(short, long, default_value = "personal-developer")]
    profile: String,

    /// AWS region
    #[arg(short, long, default_value = "us-east-1")]
    region: String,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get information from the pipeline
    Get {
        #[command(subcommand)]
        resource: GetCommands,
    },
    /// Process an audio file
    Process {
        /// S3 bucket to use
        bucket: String,

        /// Path to the audio file
        file: PathBuf,

        /// Language code (e.g., en-US, es-ES, fr-FR)
        #[arg(long, default_value = "en-US")]
        language: String,

        /// Wait for processing to complete
        #[arg(long)]
        wait: bool,

        /// Output file path for the report
        #[arg(long)]
        report_output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GetCommands {
    /// List available S3 buckets
    Buckets,
    /// Get status of a processed file
    Status {
        /// S3 bucket containing the file
        bucket: String,

        /// File key in S3
        file: String,
    },
    /// Get the analysis report
    Report {
        /// S3 bucket containing the report
        bucket: String,

        /// Original file key
        file: String,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Get the transcript
    Transcript {
        /// S3 bucket containing the transcript
        bucket: String,

        /// Original file key
        file: String,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

struct Pipeline {
    s3_client: aws_sdk_s3::Client,
    sfn_client: SfnClient,
    state_machine_arn: String,
}

impl Pipeline {
    async fn new(profile: String, region: String) -> Result<Self> {
        let config = aws_config::from_env()
            .profile_name(profile)
            .region(Region::new(region))
            .load()
            .await;

        // TODO: Make this configurable
        let state_machine_arn =
            "arn:aws:states:us-east-1:816069165876:stateMachine:AudioProcessingPipeline"
                .to_string();

        Ok(Self {
            s3_client: aws_sdk_s3::Client::new(&config),
            sfn_client: SfnClient::new(&config),
            state_machine_arn,
        })
    }

    async fn get_buckets(&self) -> Result<()> {
        let buckets = self.s3_client.list_buckets().send().await?;

        for bucket in buckets.buckets().unwrap_or_default() {
            println!("{}", bucket.name().unwrap_or_default());
        }

        Ok(())
    }

    async fn process_file(
        &self,
        file: PathBuf,
        bucket: &str,
        language: &str,
        wait: bool,
        report_output: Option<PathBuf>,
    ) -> Result<()> {
        // Validate file exists
        if !file.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", file));
        }

        // Upload file to S3
        let filename = file
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy();

        let key = filename.to_string();

        println!("📤 Uploading file to S3...");
        let content = fs::read(&file).await?;
        self.s3_client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(content.into())
            .send()
            .await?;

        println!("✅ File uploaded successfully");
        println!("🔑 File key: {}", key);

        // Start Step Function execution
        println!("\n🚀 Starting audio processing pipeline...");
        let input = serde_json::json!({
            "bucket": bucket,
            "key": key,
            "languageCode": language
        });

        let execution = self
            .sfn_client
            .start_execution()
            .state_machine_arn(&self.state_machine_arn)
            .input(serde_json::to_string(&input)?)
            .send()
            .await?;

        let execution_arn = execution.execution_arn().unwrap_or_default();
        println!("✨ Processing started!");
        println!("📋 Execution ARN: {}", execution_arn);

        if wait {
            println!("\n⏳ Waiting for processing to complete...");
            self.await_execution(&execution_arn).await?;
            println!("\n✅ Processing complete!");

            // Get and display the report
            self.get_report(bucket, &key, report_output).await?;
        }

        Ok(())
    }

    async fn await_execution(&self, execution_arn: &str) -> Result<()> {
        loop {
            let execution = self
                .sfn_client
                .describe_execution()
                .execution_arn(execution_arn)
                .send()
                .await?;

            match execution.status() {
                Some(ExecutionStatus::Running) => {
                    print!(".");
                    sleep(Duration::from_secs(5)).await;
                }
                Some(ExecutionStatus::Succeeded) => {
                    println!("\n🎉 Execution completed successfully!");
                    break;
                }
                Some(ExecutionStatus::Failed) => {
                    return Err(anyhow::anyhow!(
                        "Execution failed: {}",
                        execution.error().unwrap_or_default()
                    ));
                }
                Some(status) => {
                    return Err(anyhow::anyhow!("Unexpected status: {:?}", status));
                }
                None => {
                    return Err(anyhow::anyhow!("Could not determine execution status"));
                }
            }
        }
        Ok(())
    }

    async fn get_execution_for_file(&self, file_key: &str) -> Result<String> {
        let executions = self
            .sfn_client
            .list_executions()
            .state_machine_arn(&self.state_machine_arn)
            .send()
            .await?;

        for execution in executions.executions().unwrap_or_default() {
            // Get the execution details to access input
            let execution_details = self
                .sfn_client
                .describe_execution()
                .execution_arn(execution.execution_arn().unwrap_or_default())
                .send()
                .await?;

            if let Some(input) = execution_details.input() {
                if let Ok(input) = serde_json::from_str::<serde_json::Value>(input) {
                    if input["key"].as_str() == Some(file_key) {
                        return Ok(execution.execution_arn().unwrap_or_default().to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No execution found for file: {}", file_key))
    }

    async fn get_status(&self, bucket: &str, file_key: &str) -> Result<()> {
        let execution_arn = self.get_execution_for_file(file_key).await?;
        let execution = self
            .sfn_client
            .describe_execution()
            .execution_arn(&execution_arn)
            .send()
            .await?;

        println!("📊 Status Report");
        println!("---------------");
        println!("File: {}", file_key);
        println!("Bucket: {}", bucket);
        println!(
            "Status: {}",
            execution.status().expect("that status exists.").as_str()
        );
        println!(
            "Started: {}",
            execution
                .start_date()
                .expect("that start date exists.")
                .fmt(aws_sdk_s3::primitives::DateTimeFormat::HttpDate)?
        );

        if let Some(stop_date) = execution.stop_date() {
            println!("Completed: {}", stop_date.fmt(DateTimeFormat::HttpDate)?);
        }

        if let Some(error) = execution.error() {
            println!("\n❌ Error: {}", error);
        }

        Ok(())
    }

    async fn get_transcript(&self, bucket: &str, key: &str, output: Option<PathBuf>) -> Result<()> {
        let transcript_key = format!("{}-transcript.json", key);

        let transcript = self
            .s3_client
            .get_object()
            .bucket(bucket)
            .key(&transcript_key)
            .send()
            .await?;

        let content = transcript.body.collect().await?;
        let transcript_json: serde_json::Value = serde_json::from_slice(&content.to_vec())?;

        // Extract just the transcript text
        let transcript_text = transcript_json["results"]["transcripts"][0]["transcript"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract transcript text"))?;

        match output {
            Some(path) => {
                fs::write(&path, transcript_text).await?;
                println!("📝 Transcript saved to {:?}", path);
            }
            None => {
                println!("{}", transcript_text);
            }
        }

        Ok(())
    }

    async fn get_report(&self, bucket: &str, key: &str, output: Option<PathBuf>) -> Result<()> {
        let report_key = format!("{}-report.md", key);

        let report = self
            .s3_client
            .get_object()
            .bucket(bucket)
            .key(&report_key)
            .send()
            .await?;

        let content = report.body.collect().await?;
        let report_text = String::from_utf8(content.to_vec())?;

        assert!(
            report_text.starts_with('"') && report_text.ends_with('"'),
            "Expected report text to be wrapped in quotes: {}",
            report_text
        );

        // format the report text
        let report_text = &report_text[1..report_text.len() - 1].replace("\\n", "\n");

        match output {
            Some(path) => {
                fs::write(&path, &report_text).await?;
                println!("📝 Report saved to {:?}", path);
            }
            None => {
                println!("{}", report_text);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        profile,
        region,
        verbose,
        command,
    } = Args::parse();

    if verbose {
        tracing_subscriber::fmt::init();
    }

    let pipeline = Pipeline::new(profile, region).await?;

    match command {
        Commands::Get { resource } => match resource {
            GetCommands::Buckets => {
                pipeline.get_buckets().await?;
            }
            GetCommands::Status { bucket, file } => {
                pipeline.get_status(&bucket, &file).await?;
            }
            GetCommands::Transcript {
                bucket,
                file,
                output,
            } => {
                pipeline.get_transcript(&bucket, &file, output).await?;
            }
            GetCommands::Report {
                bucket,
                file,
                output,
            } => {
                pipeline.get_report(&bucket, &file, output).await?;
            }
        },
        Commands::Process {
            bucket,
            file,
            language,
            wait,
            report_output,
        } => {
            pipeline
                .process_file(file, &bucket, &language, wait, report_output)
                .await?;
        }
    }

    Ok(())
}
