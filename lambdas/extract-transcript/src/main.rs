use anyhow::Result;
use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use text_splitter::TextSplitter;

const CHUNK_SIZE_RANGE: Range<usize> = 4500..4900; // Slightly under 5000 to be safe

#[derive(Debug, Deserialize)]
struct TranscriptInput {
    bucket: String,
    key: String,
}

#[derive(Debug, Serialize)]
struct TranscriptOutput {
    full_text: String,
    chunks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TranscribeOutput {
    results: TranscribeResults,
}

#[derive(Debug, Deserialize)]
struct TranscribeResults {
    transcripts: Vec<Transcript>,
}

#[derive(Debug, Deserialize)]
struct Transcript {
    transcript: String,
}

#[derive(Debug, Serialize)]
struct Response {
    statusCode: i32,
    body: TranscriptOutput,
}

async fn function_handler(event: LambdaEvent<TranscriptInput>) -> Result<Response, Error> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    // Get the file from S3
    let output = s3_client
        .get_object()
        .bucket(&event.payload.bucket)
        .key(&event.payload.key)
        .send()
        .await
        .map_err(|e| Error::from(format!("Failed to get S3 object: {}", e)))?;

    // Read the body to bytes
    let body = output
        .body
        .collect()
        .await
        .map_err(|e| Error::from(format!("Failed to read S3 object body: {}", e)))?
        .into_bytes();

    // Parse the JSON content
    let transcript: TranscribeOutput = serde_json::from_slice(&body)
        .map_err(|e| Error::from(format!("Failed to parse transcript JSON: {}", e)))?;

    let full_text = transcript
        .results
        .transcripts
        .first()
        .map(|t| t.transcript.clone())
        .unwrap_or_default();

    let chunks = TextSplitter::new(CHUNK_SIZE_RANGE)
        .chunks(&full_text)
        .map(|c| c.to_string())
        .collect();

    let transcript_output = TranscriptOutput { full_text, chunks };

    Ok(Response {
        statusCode: 200,
        body: transcript_output,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    run(service_fn(function_handler)).await
}
