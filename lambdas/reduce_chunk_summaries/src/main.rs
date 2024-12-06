use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ChunkResults {
    chunkResults: Vec<ChunkAnalysis>,
}

#[derive(Debug, Deserialize)]
struct ChunkAnalysis {
    chunkAnalysis: Vec<BedrockWrapper>,
}

#[derive(Debug, Deserialize)]
struct BedrockWrapper {
    Body: BedrockMessage,
}

#[derive(Debug, Deserialize)]
struct BedrockMessage {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct CombinedOutput {
    summaries: String,
    topics: String,
}

#[derive(Debug, Serialize)]
struct Response {
    statusCode: i32,
    body: CombinedOutput,
}

fn extract_text(wrapper: &BedrockWrapper) -> String {
    wrapper
        .Body
        .content
        .iter()
        .filter(|block| block.content_type == "text")
        .map(|block| block.text.clone())
        .collect::<Vec<String>>()
        .join("\n")
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
    let chunk_results: ChunkResults = serde_json::from_value(event.payload)
        .map_err(|e| Error::from(format!("Failed to parse input payload: {}", e)))?;

    // Combine summaries with section numbers
    let summaries = chunk_results
        .chunkResults
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let summary = extract_text(&chunk.chunkAnalysis[0]);
            format!("Section {}:\n{}", i + 1, summary.trim())
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    // Combine topics with section numbers
    let topics = chunk_results
        .chunkResults
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let topics = extract_text(&chunk.chunkAnalysis[1]);
            format!("Section {}:\n{}", i + 1, topics.trim())
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    Ok(Response {
        statusCode: 200,
        body: CombinedOutput { summaries, topics },
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
