# Lambda Functions

This directory contains Lambda functions used in the audio processing pipeline.

## Functions

### compile_text_analysis

A Rust-based Lambda function that compiles various text analysis results into a
formatted Markdown report. See the function's README for detailed documentation.

## Development Guidelines

- Use infrastructure as code (preferably AWS CDK or Terraform)
- Implement comprehensive error handling
- Include appropriate logging
- Follow least-privilege principle for IAM roles
- Add unit tests and integration tests where applicable

## Deployment

Each Lambda function includes its own build and deployment instructions.
Generally:

1. Build the function following its README
2. Deploy using your preferred IaC tool
3. Verify the deployment
4. Update the Step Functions definition if needed
