use anyhow::Result;
use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

const MAX_CHUNK_SIZE: usize = 4900; // Slightly under 5000 to be safe

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

fn chunk_text(text: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_size = 0;

    // Split by sentences (roughly) to keep context
    for sentence in text.split(['.', '!', '?']) {
        let sentence = sentence.trim();
        if sentence.is_empty() {
            continue;
        }

        let sentence_bytes = sentence.as_bytes().len();

        // If single sentence is bigger than max size, split by words
        if sentence_bytes > max_size {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_size = 0;
            }

            let mut word_chunk = String::new();
            for word in sentence.split_whitespace() {
                let word_bytes = word.as_bytes().len() + 1; // +1 for space
                if word_chunk.as_bytes().len() + word_bytes > max_size && !word_chunk.is_empty() {
                    chunks.push(word_chunk.clone());
                    word_chunk.clear();
                }
                if !word_chunk.is_empty() {
                    word_chunk.push(' ');
                }
                word_chunk.push_str(word);
            }
            if !word_chunk.is_empty() {
                chunks.push(word_chunk);
            }
            continue;
        }

        // Check if adding this sentence would exceed max size
        if current_size + sentence_bytes + 2 > max_size {
            // +2 for ". "
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_size = 0;
            }
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str(". ");
            current_size += 2;
        }
        current_chunk.push_str(sentence);
        current_size += sentence_bytes;
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
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

    let chunks = chunk_text(&full_text, MAX_CHUNK_SIZE);

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
