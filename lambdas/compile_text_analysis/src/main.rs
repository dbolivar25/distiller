use anyhow::Result;
use chrono::Utc;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AnalysisResult {
    summary: BedrockWrapper,
    sentiment: SentimentData,
    entities: EntityData,
    topics: BedrockWrapper,
}

#[derive(Debug, Deserialize)]
struct BedrockWrapper {
    Body: BedrockMessage,
    ContentType: String,
}

#[derive(Debug, Deserialize)]
struct BedrockMessage {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    role: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: String,
    #[serde(rename = "stop_sequence")]
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: i32,
    output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct SentimentData {
    Sentiment: String,
    SentimentScore: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct Entity {
    Text: String,
    Type: String,
    Score: f64,
    BeginOffset: i32,
    EndOffset: i32,
}

#[derive(Debug, Deserialize)]
struct EntityData {
    Entities: Vec<Entity>,
}

#[derive(Serialize)]
struct Response {
    statusCode: i32,
    body: String,
    headers: HashMap<String, String>,
}

fn extract_text(bedrock_response: &BedrockWrapper) -> String {
    bedrock_response
        .Body
        .content
        .iter()
        .filter(|block| block.content_type == "text")
        .map(|block| block.text.clone())
        .collect::<Vec<String>>()
        .join("\n")
}

fn format_sentiment_score(score: f64) -> String {
    format!("{:.1}%", score * 100.0)
}

fn get_sentiment_emoji(sentiment: &str) -> &'static str {
    match sentiment {
        "POSITIVE" => "😊",
        "NEGATIVE" => "😔",
        "NEUTRAL" => "😐",
        "MIXED" => "🤔",
        _ => "",
    }
}

fn format_entities(entities: &[Entity]) -> String {
    if entities.is_empty() {
        return String::from("No entities detected");
    }

    let mut entity_groups: HashMap<String, Vec<&Entity>> = HashMap::new();

    for entity in entities {
        entity_groups
            .entry(entity.Type.clone())
            .or_default()
            .push(entity);
    }

    let mut sections = Vec::new();
    for (entity_type, entities) in entity_groups {
        let entity_texts: Vec<String> = entities
            .iter()
            .map(|entity| {
                format!(
                    "- {} (confidence: {:.1}%)",
                    entity.Text,
                    entity.Score * 100.0
                )
            })
            .collect();

        sections.push(format!("### {}\n{}", entity_type, entity_texts.join("\n")));
    }

    sections.join("\n\n")
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
    tracing::info!(
        "Input payload: {}",
        serde_json::to_string_pretty(&event.payload)?
    );

    let analysis: AnalysisResult = serde_json::from_value(event.payload.clone())
        .map_err(|e| Error::from(format!("Failed to parse Step Functions payload: {}", e)))?;

    let key = event
        .payload
        .get("key")
        .and_then(|k| k.as_str())
        .unwrap_or("unknown");

    let summary = extract_text(&analysis.summary);
    let topics = extract_text(&analysis.topics);
    let sentiment_emoji = get_sentiment_emoji(&analysis.sentiment.Sentiment);

    let markdown = format!(
        r#"# Analysis Results for {}
Generated on {} UTC

{}

{}

## Sentiment Analysis {}
Overall sentiment: **{}**

Confidence Scores:
- Positive: {}
- Negative: {}
- Neutral: {}
- Mixed: {}

## Named Entities
{}
"#,
        key,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        summary.replace("# ", "## "),
        topics.replace("# ", "## "),
        sentiment_emoji,
        analysis.sentiment.Sentiment.to_lowercase(),
        format_sentiment_score(
            *analysis
                .sentiment
                .SentimentScore
                .get("Positive")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *analysis
                .sentiment
                .SentimentScore
                .get("Negative")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *analysis
                .sentiment
                .SentimentScore
                .get("Neutral")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *analysis
                .sentiment
                .SentimentScore
                .get("Mixed")
                .unwrap_or(&0.0)
        ),
        format_entities(&analysis.entities.Entities)
    );

    let mut headers = HashMap::new();
    headers.insert(String::from("Content-Type"), String::from("text/markdown"));

    Ok(Response {
        statusCode: 200,
        body: markdown,
        headers,
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
