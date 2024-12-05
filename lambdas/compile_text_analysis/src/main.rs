use anyhow::Result;
use chrono::Utc;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AnalysisResult {
    overview: BedrockWrapper,
    main_topics: BedrockWrapper,
    chunk_summaries: Vec<ChunkAnalysis>,
    sentiment: Vec<Vec<SentimentData>>,
    entities: Vec<Vec<EntityData>>,
    key: String,
}

#[derive(Debug, Deserialize)]
struct ChunkAnalysis {
    chunkAnalysis: Vec<BedrockWrapper>,
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

#[derive(Debug, Deserialize, Clone)]
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

fn combine_sentiment_data(sentiments: &[Vec<SentimentData>]) -> SentimentData {
    // Flatten the nested arrays
    let flattened: Vec<&SentimentData> = sentiments.iter().flat_map(|inner| inner.iter()).collect();

    if flattened.is_empty() {
        return SentimentData {
            Sentiment: "NEUTRAL".to_string(),
            SentimentScore: HashMap::new(),
        };
    }

    // Count occurrences of each sentiment
    let mut sentiment_counts: HashMap<String, usize> = HashMap::new();
    let mut total_scores: HashMap<String, f64> = HashMap::new();
    let sentiment_count = flattened.len() as f64;

    for sentiment in flattened {
        *sentiment_counts
            .entry(sentiment.Sentiment.clone())
            .or_default() += 1;

        for (key, &score) in &sentiment.SentimentScore {
            *total_scores.entry(key.clone()).or_default() += score;
        }
    }

    // Find the most common sentiment
    let overall_sentiment = sentiment_counts
        .iter()
        .max_by_key(|&(_, count)| *count)
        .map(|(sentiment, _)| sentiment.clone())
        .unwrap_or_else(|| "NEUTRAL".to_string());

    // Average the scores
    let mut average_scores = HashMap::new();
    for (key, total) in total_scores {
        average_scores.insert(key, total / sentiment_count);
    }

    SentimentData {
        Sentiment: overall_sentiment,
        SentimentScore: average_scores,
    }
}

fn format_entities(entities_chunks: &[Vec<EntityData>]) -> String {
    // Flatten and collect all entities
    let all_entities: Vec<&Entity> = entities_chunks
        .iter()
        .flat_map(|chunk| chunk.iter())
        .flat_map(|data| data.Entities.iter())
        .collect();

    if all_entities.is_empty() {
        return String::from("No entities detected");
    }

    // Deduplicate entities by text and type, keeping the highest confidence score
    let mut unique_entities: HashMap<(String, String), &Entity> = HashMap::new();
    for entity in all_entities.iter() {
        unique_entities
            .entry((entity.Text.clone(), entity.Type.clone()))
            .and_modify(|e| {
                if entity.Score > e.Score {
                    *e = entity;
                }
            })
            .or_insert(entity);
    }

    // Group by entity type
    let mut entity_groups: HashMap<String, Vec<&Entity>> = HashMap::new();
    for entity in unique_entities.values() {
        entity_groups
            .entry(entity.Type.clone())
            .or_default()
            .push(entity);
    }

    let mut sections = Vec::new();
    for (entity_type, entities) in entity_groups {
        // Sort entities by confidence score
        let mut entities = entities.to_vec();
        entities.sort_by(|a, b| {
            b.Score
                .partial_cmp(&a.Score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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

fn format_chunk_summaries(chunk_summaries: &[ChunkAnalysis]) -> String {
    chunk_summaries
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let summary = extract_text(&chunk.chunkAnalysis[0]);
            let topics = extract_text(&chunk.chunkAnalysis[1]);
            format!(
                "### Chunk {} Summary\n{}\n\n#### Topics\n{}",
                i + 1,
                summary.trim(),
                topics.trim()
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
    let analysis: AnalysisResult = serde_json::from_value(event.payload)?;

    let overview = extract_text(&analysis.overview);
    let main_topics = extract_text(&analysis.main_topics);
    let combined_sentiment = combine_sentiment_data(&analysis.sentiment);
    let sentiment_emoji = get_sentiment_emoji(&combined_sentiment.Sentiment);

    let markdown = format!(
        r#"# Analysis Results for {}
Generated on {} UTC

## Overview
{}

## Main Topics
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

## Detailed Section Summaries
{}
"#,
        analysis.key,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        overview,
        main_topics,
        sentiment_emoji,
        combined_sentiment.Sentiment.to_lowercase(),
        format_sentiment_score(
            *combined_sentiment
                .SentimentScore
                .get("Positive")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *combined_sentiment
                .SentimentScore
                .get("Negative")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *combined_sentiment
                .SentimentScore
                .get("Neutral")
                .unwrap_or(&0.0)
        ),
        format_sentiment_score(
            *combined_sentiment
                .SentimentScore
                .get("Mixed")
                .unwrap_or(&0.0)
        ),
        format_entities(&analysis.entities),
        format_chunk_summaries(&analysis.chunk_summaries)
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
