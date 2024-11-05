# Text Analysis Compilation Lambda

This Lambda function is responsible for compiling various text analysis results
into a comprehensive Markdown report. It's written in Rust for optimal
performance and memory efficiency.

## Features

- Processes analysis results from multiple sources:
  - Claude (via Bedrock) text summaries and topic detection
  - Amazon Comprehend entity detection and sentiment analysis
- Generates formatted Markdown reports with:
  - Content summaries
  - Topic listings
  - Named entity recognition results
  - Sentiment analysis with confidence scores
- Includes metadata and timestamps in the output

## Development

This project uses Rust 2021 edition and the following key dependencies:

- `lambda_runtime`: AWS Lambda Runtime for Rust
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `chrono`: DateTime handling
- `tracing`: Logging and diagnostics

### Building

```bash
cargo lambda build --release --arm64
```

### Deployment

The Lambda is optimized for size and performance with:

- Strip symbols enabled
- Link-time optimization
- Minimal codegen units
- Panic=abort for smaller binary size

## Configuration

The function expects input in a specific JSON format containing:

- Text summary from Claude
- Topic analysis from Claude
- Entity detection results from Comprehend
- Sentiment analysis results from Comprehend
