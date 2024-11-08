# Step Functions Workflows

This directory contains AWS Step Functions state machine definitions for audio
processing pipelines.

## Workflows

### AudioProcessingPipeline

A comprehensive pipeline that processes audio files through multiple AWS
services:

- Amazon Transcribe for speech-to-text
- Amazon Comprehend for entity and sentiment analysis
- Amazon Bedrock (Claude) for summarization and topic detection
- Lambda for result compilation

#### Input Parameters

```json
{
  "bucket": "string", // S3 bucket containing the audio file
  "key": "string", // S3 key for the audio file
  "languageCode": "string" // Language code (e.g., "en-US")
}
```

#### Output

- Generates a comprehensive Markdown report
- Stores results in the same S3 bucket with "-report.md" suffix

## Deployment

1. Deploy required Lambda functions first
2. Create/update the state machine using AWS CLI or Console
3. Verify IAM roles and permissions
4. Test with sample audio files
