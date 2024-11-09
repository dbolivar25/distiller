use crate::{
    args::ProcessArgs,
    display::{
        print_divider, print_error, print_header, print_success, print_table_row, truncate_arn,
    },
};
use anyhow::{bail, Context, Result};
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sfn::{types::ExecutionStatus, Client as SfnClient};
use console::{style, StyledObject};
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};
use tokio::{fs, time::sleep};
use tracing::debug;

const DEFAULT_LANGUAGE: &str = "en-US";
const STATE_MACHINE_ARN: &str =
    "arn:aws:states:us-east-1:816069165876:stateMachine:AudioProcessingPipeline";
const POLLING_INTERVAL: u64 = 5;
const SPINNER_INTERVAL: u64 = 100;
const SPINNER_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
const DIVIDER_WIDTH: usize = 60;

pub(crate) struct Client {
    s3_client: S3Client,
    sfn_client: SfnClient,
}

impl Client {
    pub(crate) async fn new(profile: Option<String>, region: Option<String>) -> Self {
        let config = match (profile, region) {
            (Some(profile), Some(region)) => aws_config::from_env()
                .profile_name(profile)
                .region(Region::new(region)),
            (Some(profile), None) => aws_config::from_env().profile_name(profile),
            (None, Some(region)) => aws_config::from_env().region(Region::new(region)),
            _ => aws_config::from_env(),
        }
        .load()
        .await;

        debug!("AWS config initialized: {:?}", config);

        Self {
            s3_client: S3Client::new(&config),
            sfn_client: SfnClient::new(&config),
        }
    }

    pub(crate) async fn list_buckets(&self) -> Result<()> {
        let buckets = self
            .s3_client
            .list_buckets()
            .send()
            .await
            .context("Failed to list buckets")?;

        print_header("Available Buckets");
        print_divider();

        for (i, bucket) in buckets.buckets().into_iter().flatten().enumerate() {
            if let Some(name) = bucket.name() {
                print_table_row(&format!("{}", i + 1), name);
            }
        }

        print_divider();

        Ok(())
    }

    pub(crate) async fn get_status(&self, bucket: &str, key: &str) -> Result<()> {
        let execution_arn = self.find_execution(key).await?;
        let execution = self
            .sfn_client
            .describe_execution()
            .execution_arn(&execution_arn)
            .send()
            .await
            .context("Failed to get execution description")?;

        print_header("Status Report");
        print_divider();
        print_table_row("File:", key);
        print_table_row("Bucket:", bucket);
        print_table_row(
            "Status:",
            get_status_style(execution.status().expect("execution should have status")).to_string(),
        );
        print_table_row("Execution:", truncate_arn(&execution_arn));
        print_divider();

        if let Some(error) = execution.error() {
            print_error(error);
            tracing::error!("{}", error);
        }

        Ok(())
    }

    async fn find_execution(&self, file_key: &str) -> Result<String> {
        let executions = self
            .sfn_client
            .list_executions()
            .state_machine_arn(STATE_MACHINE_ARN)
            .send()
            .await
            .context("Failed to list executions")?;

        for execution in executions.executions().unwrap_or_default() {
            if let Some(input) = self
                .sfn_client
                .describe_execution()
                .execution_arn(execution.execution_arn().unwrap_or_default())
                .send()
                .await
                .ok()
                .and_then(|e| e.input().map(String::from))
            {
                if let Ok(input) = serde_json::from_str::<serde_json::Value>(&input) {
                    if input["key"].as_str() == Some(file_key) {
                        return Ok(execution.execution_arn().unwrap_or_default().to_string());
                    }
                }
            }
        }

        bail!("No execution found for file: {}", file_key)
    }

    pub(crate) async fn process_file(&self, args: ProcessArgs) -> Result<()> {
        let ProcessArgs {
            bucket,
            file,
            language,
            transcript_output,
            report_output,
            ..
        } = args;

        if !file.exists() {
            bail!("File does not exist: {:?}", file);
        }

        let key = file
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy()
            .to_string();

        print_header("Processing Job");
        print_divider();
        print_table_row("File:", &key);
        print_table_row("Bucket:", &bucket);
        print_table_row("Language:", language.as_deref().unwrap_or(DEFAULT_LANGUAGE));
        print_divider();

        self.s3_client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(fs::read(&file).await?.into())
            .send()
            .await
            .context("Failed to upload file")?;

        let execution = self
            .sfn_client
            .start_execution()
            .state_machine_arn(STATE_MACHINE_ARN)
            .input(serde_json::to_string(&serde_json::json!({
                "bucket": bucket,
                "key": key,
                "languageCode": language.unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
            }))?)
            .send()
            .await
            .context("Failed to start execution")?;

        let execution_arn = execution.execution_arn().context("Missing execution ARN")?;

        if args.wait {
            self.wait_for_completion(execution_arn).await?;

            self.get_transcript(&bucket, &key, transcript_output)
                .await?;
            self.get_report(&bucket, &key, report_output).await?;
        }

        Ok(())
    }

    async fn wait_for_completion(&self, execution_arn: &str) -> Result<()> {
        let progress_style = ProgressStyle::default_spinner()
            .tick_chars(SPINNER_CHARS)
            .template(&format!(
                "{{spinner:.blue}} {{prefix:.bold.dim}} {{msg}} [{{elapsed_precise}}]{}",
                " ".repeat(DIVIDER_WIDTH - 50)
            ))
            .context("Failed to create progress style")?;

        let progress = ProgressBar::new_spinner().with_style(progress_style);

        progress.enable_steady_tick(Duration::from_millis(SPINNER_INTERVAL));
        progress.set_prefix("Processing");
        progress.set_message("initializing...");

        loop {
            let execution = self
                .sfn_client
                .describe_execution()
                .execution_arn(execution_arn)
                .send()
                .await
                .context("Failed to get execution")?;

            match execution
                .status()
                .expect("execution description should have a status")
            {
                ExecutionStatus::Running => {
                    progress
                        .set_message(truncate_arn(execution.execution_arn().unwrap_or_default()));
                    sleep(Duration::from_secs(POLLING_INTERVAL)).await;
                }
                ExecutionStatus::Succeeded => {
                    progress.finish_with_message(format!("{}", style("complete").green().bold()));
                    print_divider();
                    break;
                }
                ExecutionStatus::Failed => {
                    progress.finish_with_message(format!("{}", style("failed").red().bold()));
                    print_divider();
                    bail!("Execution failed");
                }
                status => bail!("Unexpected status: {:?}", status),
            }
        }

        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<bytes::Bytes> {
        Ok(self
            .s3_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?
            .body
            .collect()
            .await?
            .into_bytes())
    }

    pub(crate) async fn get_transcript(
        &self,
        bucket: &str,
        key: &str,
        output: Option<PathBuf>,
    ) -> Result<()> {
        let file_path = PathBuf::from(key);
        let filename = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid key format"))?;

        let transcript_key = format!("{}-transcript.json", filename.to_string_lossy());
        let content = self.get_object(bucket, &transcript_key).await?;

        let transcript_json = serde_json::from_slice::<serde_json::Value>(&content)?;

        let transcript = transcript_json
            .get("results")
            .and_then(|r| r.get("transcripts"))
            .and_then(|t| t.get(0))
            .and_then(|t| t.get("transcript"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid transcript format"))?;

        match output {
            Some(path) => {
                fs::write(&path, transcript).await?;
                print_success(format!("Transcript saved to {:?}", path));
            }
            None => {
                print_header("Transcript");
                print_divider();
                println!("{}", transcript);
                print_divider();
            }
        }

        Ok(())
    }

    pub(crate) async fn get_report(
        &self,
        bucket: &str,
        key: &str,
        output: Option<PathBuf>,
    ) -> Result<()> {
        let file_path = PathBuf::from(key);
        let filename = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid key format"))?;

        let transcript_key = format!("{}-report.md", filename.to_string_lossy());
        let content = self.get_object(bucket, &transcript_key).await?;

        let report = String::from_utf8(content.to_vec())?
            .trim_matches('"')
            .replace("\\n", "\n")
            .replace("\\\"", "\"");

        match output {
            Some(path) => {
                fs::write(&path, report).await?;
                print_success(format!("Report saved to {:?}", path));
            }
            None => {
                print_header("Report");
                print_divider();
                println!("{}", report);
                print_divider();
            }
        }

        Ok(())
    }
}

fn get_status_style(status: &ExecutionStatus) -> StyledObject<String> {
    match status {
        ExecutionStatus::Running => style(status.as_str().to_string()).yellow().bold(),
        ExecutionStatus::Succeeded => style(status.as_str().to_string()).green().bold(),
        ExecutionStatus::Failed => style(status.as_str().to_string()).red().bold(),
        _ => style(status.as_str().to_string()).dim().bold(),
    }
}
