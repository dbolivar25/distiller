# Distiller CLI

A command-line interface for interacting with the Distiller audio processing
pipeline on AWS. Built with Rust, this CLI tool provides a seamless way to
process, monitor, and retrieve results from audio analysis jobs.

## Features

- Upload and process audio files through AWS Step Functions workflow
- Monitor job status and execution progress
- Retrieve transcripts and analysis reports
- List available S3 buckets
- Support for custom AWS profiles and regions
- Configurable verbosity levels for debugging
- Progress tracking with elegant console output

## Installation

### Prerequisites

- Rust toolchain (latest stable version)
- AWS credentials configured (`~/.aws/credentials` or environment variables)
- Appropriate AWS IAM permissions for S3 and Step Functions access

### Building from Source

```bash
cargo build --release
```

The binary will be available at `target/release/distiller`

## Usage

### Help

```bash
Distiller CLI: Summarize and Analyze Meeting Audio

Usage: distiller [OPTIONS] <COMMAND>

Commands:
  get      Get resources and information from AWS
  process  Run the pipeline on a meeting audio file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

GLOBAL OPTIONS:
  -p, --profile <PROFILE>  Override the AWS profile in your environment
  -r, --region <REGION>    Override the AWS region in your environment
  -v, --verbose...         Increase the verbosity of the output. Warning: this can affect performance
```

### Commands

#### List Available Buckets

```bash
distiller get buckets
```

#### Process an Audio File

```bash
distiller process <BUCKET> <FILE>
```

#### Check Job Status

```bash
distiller get status <BUCKET> <KEY>
```

#### Retrieve Results

Get transcript:

```bash
distiller get transcript <BUCKET> <KEY>
```

Get analysis report:

```bash
distiller get report <BUCKET> <KEY>
```

## Examples

Process an audio file and wait for results:

```bash
distiller process my-bucket ./meeting.mp3 --wait --transcript-output ./transcript.txt --report-output ./report.md
```

Check status of a processing job:

```bash
distiller get status my-bucket meeting.mp3
```

List available buckets with debug output:

```bash
distiller -vvv get buckets
```

## Error Handling

The CLI provides clear error messages with:

- Descriptive error text
- Visual indicators (✅ for success, ❌ for errors)
- Optional verbose logging for debugging

## Environment Variables

- `AWS_PROFILE`: Default AWS profile (can be overridden with `--profile`)
- `AWS_REGION`: Default AWS region (can be overridden with `--region`)
- Standard AWS credential environment variables are supported
