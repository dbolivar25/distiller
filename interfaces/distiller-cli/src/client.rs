use anyhow::{bail, Context, Result};
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sfn::{types::ExecutionStatus, Client as SfnClient};
use console::style;
use std::{path::PathBuf, time::Duration};
use tokio::{fs, time::sleep};
use tracing::{debug, info};

use crate::args::ProcessArgs;

// const DEFAULT_REGION: &str = "us-east-1";
const DEFAULT_LANGUAGE: &str = "en-US";
const STATE_MACHINE_ARN: &str =
    "arn:aws:states:us-east-1:816069165876:stateMachine:AudioProcessingPipeline";
const POLLING_INTERVAL: u64 = 5;

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

        for bucket in buckets.buckets().unwrap_or_default() {
            if let Some(name) = bucket.name() {
                println!("{}", name);
            }
        }

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

        println!("\n{}", style("Status Report").bold());
        println!("{}", style("-------------").dim());
        println!("File: {}", key);
        println!("Bucket: {}", bucket);
        println!(
            "Status: {}",
            style(
                execution
                    .status()
                    .expect("execution description should have a status")
                    .as_str()
            )
            .yellow()
        );
        println!("Execution ARN: {}", execution_arn);

        if let Some(error) = execution.error() {
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

        info!("Uploading {} to {}", key, bucket);

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
        info!("Started processing: {}", execution_arn);

        if args.wait {
            self.wait_for_completion(execution_arn).await?;

            self.get_transcript(&bucket, &key, transcript_output)
                .await?;

            self.get_report(&bucket, &key, report_output).await?;
        }

        Ok(())
    }

    async fn wait_for_completion(&self, execution_arn: &str) -> Result<()> {
        let progress = indicatif::ProgressBar::new_spinner();
        progress.set_message("Processing");

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
                    progress.tick();
                    sleep(Duration::from_secs(POLLING_INTERVAL)).await;
                }
                ExecutionStatus::Succeeded => {
                    progress.finish_with_message("Processing complete!");
                    break;
                }
                ExecutionStatus::Failed => {
                    progress.finish_with_message("Processing failed!");
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
        let transcript_key = format!("{}-transcript.json", key);
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
                info!("Transcript saved to {:?}", path);
            }
            None => println!("\n{}", transcript),
        }

        Ok(())
    }

    pub(crate) async fn get_report(
        &self,
        bucket: &str,
        key: &str,
        output: Option<PathBuf>,
    ) -> Result<()> {
        let report_key = format!("{}-report.md", key);
        let content = self.get_object(bucket, &report_key).await?;

        let report = String::from_utf8(content.to_vec())?
            .trim_matches('"')
            .replace("\\n", "\n")
            .replace("\\\"", "\"");

        match output {
            Some(path) => {
                fs::write(&path, report).await?;
                info!("Report saved to {:?}", path);
            }
            None => println!("\n{}", report),
        }

        Ok(())
    }
}
