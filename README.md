# Distiller: Serverless Audio Processing Pipeline

A serverless solution for processing, transcribing, and analyzing audio content using AWS services. Distiller extracts meaningful insights from audio files through transcription, sentiment analysis, named entity recognition, and AI-powered summarization.

## Features

- 🎙️ **Audio Transcription**: Converts audio to text using AWS Transcribe with speaker diarization
- 🤖 **AI Analysis**: Leverages AWS Bedrock (Claude) for summarization and topic extraction
- 📊 **Natural Language Processing**:
  - Sentiment analysis across transcript segments
  - Named entity recognition
  - Key topic identification
- 📝 **Automated Reporting**: Generates comprehensive Markdown reports with all analysis results
- 🛠️ **CLI Interface**: User-friendly command-line tool for job submission and monitoring

## Quick Demo

Watch this [demo video](https://asciinema.org/a/MAaa5ziNWtWzdQ0zdosTUNEHJ) to see the pipeline run on an excerpt from a recorded interview discussing biological technologies such as CRISPR. Skip to near the end as that is where the transcript and report are displayed.

## Technology Stack

### Core Technologies

- **Rust**: Powers all runtime components for performance and reliability

  - Zero-cost abstractions for efficient resource usage
  - Strong type system preventing common runtime errors
  - Excellent AWS Lambda cold start performance
  - Cross-platform CLI compilation

- **AWS Step Functions**: Orchestrates the serverless workflow
  - Visual workflow management
  - Built-in error handling and retry logic
  - Parallel execution for improved performance
  - State machine validation and monitoring

### AI/ML Services

- **Amazon Transcribe**: High-accuracy audio transcription

  - Multi-speaker detection and diarization
  - Custom vocabulary support
  - Timestamped word-level output

- **Amazon Bedrock (Claude)**: Advanced language model integration

  - Content summarization with high coherence
  - Topic extraction and categorization
  - Context-aware analysis

- **Amazon Comprehend**: Natural language processing
  - Real-time sentiment analysis
  - Named entity recognition
  - Topic modeling and key phrase extraction

### Text Processing

- **Semantic Text Chunking**: Intelligent text segmentation

  - Uses `text-splitter` crate for content-aware splitting
  - Maintains semantic boundaries and context
  - Prevents sentence or paragraph truncation
  - Optimizes chunk sizes for API limits while preserving meaning
  - Ensures consistent analysis quality across segments

- **Parallel Processing**: Efficient content analysis
  - Concurrent processing of text chunks
  - Load balancing across AWS services
  - Maintains ordering for final compilation

## Architecture

The solution consists of three main components:

1. **CLI Interface (`interfaces/distiller-cli`)**

   - Rust-based command-line tool for interacting with the pipeline
   - Supports file upload, job monitoring, and result retrieval
   - AWS credentials and region configuration
   - Error handling and progress visualization

2. **Step Functions Pipeline (`step_functions/AudioProcessingPipeline.asl.json`)**

   - Orchestrates the entire processing workflow
   - Handles service coordination and error management
   - Implements retry logic and parallel processing
   - State management and execution tracking

3. **Lambda Functions (`lambdas/`)**
   - `extract-transcript`:
     - Processes AWS Transcribe output
     - Implements semantic text chunking
     - Maintains context across chunk boundaries
   - `compile_text_analysis`:
     - Combines analysis results into a formatted report
     - Aggregates sentiment across chunks
     - Deduplicates and ranks entities

## Step Function Overview

![Workflow](./assets/stepfunctions_graph.svg)

## Prerequisites

- AWS Account with appropriate permissions
- Rust toolchain (2021 edition or later)
- AWS CLI configured with credentials
- The following AWS services enabled:
  - AWS Lambda
  - AWS Step Functions
  - Amazon S3
  - Amazon Transcribe
  - Amazon Comprehend
  - Amazon Bedrock (Claude model access)

## Installation

1. Build the CLI:

```bash
cd interfaces/distiller-cli
cargo build --release
```

2. Deploy AWS resources:

```bash
# Deploy using your preferred IaC tool (CloudFormation, Terraform, etc.)
# Configure environment variables and AWS permissions
```

## Usage

### Basic Commands

```bash
# Process an audio file
distiller process my-bucket my-meeting.mp3 --wait

# Check processing status
distiller get status my-bucket my-meeting.mp3

# Retrieve results
distiller get report my-bucket my-meeting.mp3 --output report.md
distiller get transcript my-bucket my-meeting.mp3 --output transcript.txt

# List available buckets
distiller get buckets
```

### CLI Options

```bash
USAGE:
    distiller [OPTIONS] <COMMAND>

COMMANDS:
    get       Get resources and information from AWS
    process   Run the pipeline on a meeting audio file
    help      Print this message or help for a subcommand

OPTIONS:
    -p, --profile    Override the AWS profile
    -r, --region     Override the AWS region
    -v, --verbose    Increase logging verbosity
    -h, --help       Print help information
```

## Output Format

The pipeline generates two main outputs:

1. **Transcript File** (`<filename>-transcript.json`):

   - Raw transcription from AWS Transcribe
   - Speaker diarization when available
   - Timestamped text segments

2. **Analysis Report** (`<filename>-report.md`):
   - Executive summary of the content
   - Main topics discussed
   - Overall sentiment analysis
   - Named entities (people, organizations, locations, etc.)
   - Confidence scores for all analyses

## Pipeline Workflow

1. **Input Validation**

   - Verifies required parameters (bucket, key, language)
   - Ensures file accessibility

2. **Transcription**

   - Submits audio for transcription
   - Monitors job progress
   - Extracts and semantically chunks text for analysis

3. **Parallel Analysis**

   - Bedrock Analysis:
     - Content summarization
     - Topic extraction
   - Comprehend Analysis:
     - Sentiment analysis per semantic chunk
     - Entity recognition with context preservation

4. **Report Generation**
   - Compiles all analysis results
   - Formats into a structured Markdown document
   - Stores in the original S3 bucket

## Error Handling

- Automatic retries for transient failures
- Detailed error reporting through CLI
- Comprehensive logging for troubleshooting
- Graceful handling of service limits and quotas

## Performance Considerations

- Parallel processing of analysis tasks
- Context-aware semantic text chunking
- Optimized Lambda configurations
- Minimal cold start impact through Rust implementation

## Security

- AWS IAM roles and policies
- S3 bucket encryption
- Secure credential management
- No data persistence outside of S3

## Monitoring and Observability

Monitor the pipeline through:

- AWS CloudWatch Logs
- Step Functions execution console
- CLI status commands with progress visualization
- Lambda function metrics and tracing
