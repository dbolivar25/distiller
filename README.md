# Distiller

A serverless solution for processing, transcribing, and analyzing audio content using AWS services. Distiller extracts meaningful insights from audio files through transcription, sentiment analysis, named entity recognition, and AI-powered summarization.

## Features

- 🎙️ **Audio Transcription**: High-quality transcription with speaker diarization using AWS Transcribe
- 🤖 **AI-Powered Analysis**:
  - Content summarization using AWS Bedrock (Claude)
  - Topic extraction and categorization
  - Semantic text chunking for optimal processing
- 📊 **Natural Language Processing**:
  - Sentiment analysis across transcript segments
  - Named entity recognition
  - Key topic identification
- 📝 **Report Generation**: Comprehensive Markdown reports containing analysis results
- 🛠️ **CLI Tool**: Rust-based command line interface for easy interaction

## Quick Demo

Watch this [demo video](https://asciinema.org/a/MAaa5ziNWtWzdQ0zdosTUNEHJ) to see the pipeline run on an excerpt from a recorded interview discussing biological technologies such as CRISPR. Skip to near the end as that is where the transcript and report are displayed.

## Architecture

### Key Innovations

#### 1. Semantic Double-Pass Chunking

Distiller employs an advanced semantic chunking strategy that preserves context and meaning across segment boundaries - a critical challenge in processing long-form content. Our double-pass approach:

First Pass:

- Initial segmentation based on optimal token sizes (4500-4900 tokens)
- Respect for natural language boundaries (sentences, paragraphs)
- Conservative break point selection to maintain context

Second Pass (Boundary Refinement):

- Analysis of chunk boundaries for semantic coherence
- Smart merging of segments that share strong contextual relationships
- Adjustment of break points to preserve complete thoughts and discussions
- Optimization for LLM context windows while maintaining semantic integrity

Benefits:

- Prevents fragmentation of key concepts
- Maintains contextual relationships across chunks
- Optimizes chunk sizes for API limits and processing efficiency
- Enables more accurate analysis and summarization

#### 2. Recursive Analysis Architecture

At the heart of Distiller is a unique recursive analysis approach that solves a critical challenge in processing long-form audio content: LLM bias towards recently processed content. Traditional approaches often result in summaries and topic lists that overemphasize the latter portions of transcripts, leading to incomplete or skewed analysis.

Our solution implements a two-phase recursive analysis:

1. **Bottom-up Analysis Phase**

   - Processed semantic chunks receive independent analysis
   - Each chunk gets summarization and topic extraction
   - This preserves important details that might be lost in a single-pass analysis

2. **Top-down Synthesis Phase**
   - Individual chunk summaries are recursively combined
   - Topics are aggregated and re-analyzed for holistic understanding
   - Final pass creates cohesive overview that maintains balance across the entire content

This approach ensures:

- Equal representation of concepts across the entire transcript
- Preservation of important details from all sections
- More nuanced topic identification
- Balanced final summaries that capture the true distribution of ideas

### Components

1. **CLI Interface** (`interfaces/distiller-cli`)

   - Rust implementation for AWS service interaction
   - File upload and job management
   - Progress tracking and result retrieval
   - Configurable AWS credentials and regions

2. **Step Functions Workflow** (`step_functions/AudioProcessingPipeline.asl.json`)

   - Orchestrates the recursive analysis pipeline
   - Manages parallel processing and error handling
   - Implements automatic retries and state tracking
   - Coordinates the bottom-up and top-down analysis phases

3. **Lambda Functions** (`lambdas/`)
   - `extract-transcript`: Implements double-pass semantic chunking with boundary refinement
   - `reduce_chunk_summaries`: Performs recursive summary combination and topic synthesis
   - `compile_text_analysis`: Generates balanced, comprehensive reports

### AWS Services Used

- AWS Lambda
- AWS Step Functions
- Amazon S3
- Amazon Transcribe
- Amazon Comprehend
- Amazon Bedrock (Claude)

## Installation

### Prerequisites

1. An AWS account with appropriate permissions
2. Rust toolchain (2021 edition or later)
3. AWS CLI configured with credentials
4. Access to required AWS services:
   - Lambda
   - Step Functions
   - S3
   - Transcribe
   - Comprehend
   - Bedrock (Claude model)

### Building the CLI

```bash
cd interfaces/distiller-cli
cargo build --release
```

## Usage

### Basic Commands

```bash
# List available S3 buckets
distiller get buckets

# Process an audio file
distiller process <bucket> <file> --wait

# Check job status
distiller get status <bucket> <key>

# Retrieve results
distiller get report <bucket> <key> --output report.md
distiller get transcript <bucket> <key> --output transcript.txt
```

### CLI Options

```
USAGE:
    distiller [OPTIONS] <COMMAND>

COMMANDS:
    get       Retrieve AWS resources and information
    process   Execute pipeline on audio file
    help      Display help information

OPTIONS:
    -p, --profile    AWS profile override
    -r, --region     AWS region override
    -v, --verbose    Increase logging detail
    -h, --help       Show help
```

### Processing Options

```bash
distiller process [OPTIONS] <BUCKET> <FILE>

OPTIONS:
    --language            Specify audio language (default: en-US)
    --wait               Wait for processing completion
    --transcript-output   Save transcript to file
    --report-output      Save report to file
```

## Pipeline Workflow

1. **Input Validation**

   - Validates required parameters
   - Verifies file accessibility

2. **Transcription**

   - Submits audio to AWS Transcribe
   - Monitors job progress
   - Extracts and chunks text

3. **Recursive Parallel Analysis**

   - Initial Bottom-up Phase:
     - Parallel processing of semantic chunks
     - Individual chunk summarization
     - Per-chunk topic extraction
     - Sentiment analysis and entity recognition
   - Synthesis Phase:
     - Recursive combination of chunk summaries
     - Topic aggregation and re-analysis
     - Generation of balanced overview
     - Cross-chunk entity correlation

4. **Results Compilation**
   - Combines bottom-up and top-down analyses
   - Ensures balanced representation of entire content
   - Generates comprehensive formatted report
   - Stores all results in S3

## Output Files

1. **Transcript** (`<filename>-transcript.json`)

   - Complete transcription
   - Speaker labels
   - Timestamp metadata

2. **Analysis Report** (`<filename>-report.md`)
   - Executive summary
   - Main topics
   - Sentiment analysis results
   - Identified entities
   - Confidence metrics

## Performance Features

- Parallel processing of analysis tasks
- Semantic text chunking for optimal processing
- Rust implementation for minimal cold starts
- Efficient resource utilization

## Error Handling

- Automatic retries for transient failures
- Detailed error reporting
- Comprehensive logging
- AWS service quota management

## Security Considerations

- IAM role-based access control
- S3 bucket encryption
- Secure credential handling
- Temporary resource persistence

## Monitoring

- CloudWatch Logs integration
- Step Functions execution tracking
- CLI status monitoring
- Lambda metrics and tracing

## Project Structure

```
.
├── assets/                 # Project assets
├── interfaces/
│   └── distiller-cli/     # Rust CLI implementation
├── lambdas/               # Lambda function implementations
│   ├── extract-transcript/
│   ├── reduce_chunk_summaries/
│   └── compile_text_analysis/
└── step_functions/        # Step Functions workflow definition
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to your branch
5. Create a Pull Request

## License

[Insert License Information]
