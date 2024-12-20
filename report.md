---
geometry: margin=1in
---

# Distiller

## Authors: Daniel Bolivar, Clint Lang, Will Owens

## Abstract

Distiller is a serverless framework for automated analysis of long-form audio
content that addresses key challenges in maintaining semantic coherence and
balanced representation across extended technical discussions. The system
employs two key techniques: semantic double-pass merging for context-preserving
text segmentation, and a recursive analysis architecture that ensures balanced
content representation across varying document lengths.

Built on AWS serverless technologies, Distiller implements a multi-stage
pipeline that combines audio transcription, natural language processing, and
AI-powered analysis through parallel processing streams. The framework's modular
architecture enables scalable processing while maintaining semantic
relationships across content boundaries.

Evaluation through complex technical discussions, particularly in genetic
engineering, demonstrates the system's effectiveness in preserving specialized
terminology, maintaining contextual relationships, and producing balanced
analyses of extended content. The framework achieves this while operating within
practical resource constraints through careful optimization of chunk sizes and
processing strategies.

Our implementation shows that combining semantic chunking with recursive
analysis enables more comprehensive and balanced content processing than
traditional approaches. The system's success in handling complex technical
discussions while maintaining both accuracy and efficiency demonstrates the
viability of automated analysis for sophisticated audio content.

## 1. Introduction

### 1.1 Background and Motivation

The analysis of audio content, particularly extended technical discussions and
interviews, presents unique challenges in both processing and understanding. As
the volume of recorded content continues to grow, organizations face increasing
difficulty in efficiently extracting insights and maintaining semantic
understanding across long-form audio materials.

#### Challenges in Audio Content Analysis

Processing extended audio content involves multiple complex tasks: accurate
transcription with speaker attribution, maintenance of semantic relationships
across segments, and extraction of key insights while preserving context.
Traditional approaches often struggle with maintaining consistent analysis
quality across technical discussions that require preservation of specialized
terminology and concepts.

#### Need for Scalable Processing Solutions

Current audio processing systems typically handle individual aspects of content
analysis—transcription, natural language processing, or summarization—as
separate concerns. This separation often leads to fragmented processing
pipelines that struggle to maintain context and semantic relationships across
processing stages. Additionally, the resource requirements for processing
long-form audio content necessitate efficient, scalable architectures that can
handle varying workloads while maintaining processing accuracy.

#### Limitations in Existing Systems

Existing solutions face several key limitations. Speaker diarization systems
often struggle with attribution in complex scenarios. Traditional content
analysis approaches tend to lose context across longer content spans. Moreover,
many current systems lack the ability to maintain balanced representation of
content when processing extended discussions, often skewing toward recent
content in their analysis.

### 1.2 Research Objectives

#### Primary Goals

The Distiller framework addresses these challenges through several key
objectives:

- Development of a scalable, serverless architecture for audio content
  processing
- Implementation of semantic chunking that preserves context across content
  boundaries
- Creation of parallel analysis streams that maintain balanced content
  representation
- Integration of multiple analysis approaches for comprehensive content
  understanding

#### Technical Challenges Addressed

Distiller specifically addresses several technical challenges:

- Maintaining semantic coherence across processing stages
- Implementing efficient resource utilization for extended content processing
- Preserving technical accuracy while enabling high-level analysis
- Ensuring consistent quality across varying content lengths and complexity

#### Target Use Cases

The framework is designed to support several key use cases:

- Processing of technical interviews and discussions
- Analysis of complex multi-speaker content
- Extraction of insights from extended audio recordings
- Generation of comprehensive content analysis reports

### 1.3 Paper Organization

This paper presents a detailed examination of the Distiller framework, organized
to provide comprehensive coverage of its design, implementation, and evaluation:

Section 2 examines related work in audio processing systems, serverless
architectures, and natural language processing, establishing the technical
foundation for Distiller's approach.

Section 3 details the system architecture, exploring the core components and
their interactions. This section provides insight into how Distiller achieves
its processing objectives through careful system design.

Section 4 delves into the technical implementation, focusing on key innovations
in semantic processing and content analysis. This section examines the specific
approaches used to address the core technical challenges.

Section 5 evaluates the system's performance through detailed examination of
real-world applications, demonstrating Distiller's effectiveness in processing
complex technical content.

The remaining section explores the conclusions of our work, providing a complete
picture of the framework's capabilities and potential.

Throughout the paper, we maintain focus on how Distiller's architectural
decisions and technical implementations address the core challenges of audio
content analysis while enabling scalable, efficient processing of complex
content.

## 2. Related Work

### 2.1 Audio Processing Systems

#### AWS Transcribe and Speech Recognition

The evolution of cloud-based speech recognition systems, particularly AWS
Transcribe, has enabled sophisticated audio processing capabilities. These
systems support critical features including speaker diarization, custom
vocabulary handling, and automatic punctuation. Current approaches to audio
transcription particularly excel in handling multiple speakers, with systems
like AWS Transcribe supporting identification and labeling of up to 10 distinct
speakers in a conversation.

#### Content Analysis Integration

Modern audio processing systems increasingly integrate with broader content
analysis capabilities. The combination of transcription services with natural
language processing has enabled more sophisticated understanding of audio
content. These integrations typically focus on key areas such as entity
recognition, sentiment analysis, and topic extraction.

#### System Limitations

Current audio processing systems face several notable limitations. Speaker
diarization, while advanced, still struggles with accurate attribution in
complex scenarios involving overlapping speech or large numbers of speakers.
Additionally, the real-time processing of audio content remains challenging,
necessitating asynchronous processing approaches for longer content.

### 2.2 Serverless Architectures

#### Current State of Serverless Computing

Serverless computing has evolved to support complex, distributed workflows
through services like AWS Step Functions. These systems enable sophisticated
orchestration of multiple processing stages while maintaining scalability and
fault tolerance. The state machine paradigm has become particularly important
for managing complex, multi-stage processing pipelines.

#### Benefits and Challenges

The serverless paradigm offers several key advantages for media processing:

- Automatic scaling to handle varying workloads
- Cost efficiency through precise resource allocation
- Built-in fault tolerance and error recovery
- Simplified deployment and maintenance

However, these systems also present specific challenges:

- Managing state across distributed components
- Handling long-running processes within function limits
- Coordinating complex workflows across multiple services
- Optimizing cold start performance

#### Media Processing Applications

Serverless architectures have shown particular utility in media processing
workflows. The ability to decompose complex processing tasks into discrete steps
allows for efficient resource utilization and parallel processing where
appropriate. The combination of function-based processing with orchestration
services enables sophisticated media processing pipelines while maintaining
operational efficiency.

### 2.3 Natural Language Processing

#### Text Chunking Approaches

Current approaches to text chunking focus on maintaining semantic coherence
while optimizing for processing efficiency. Modern systems must balance several
competing concerns:

- Maintaining natural language boundaries
- Preserving context across chunk boundaries
- Optimizing chunk sizes for downstream processing
- Handling domain-specific terminology

#### Summarization Techniques

Text summarization systems have evolved to handle increasingly complex content
through multi-stage processing:

- Initial semantic analysis
- Topic identification and extraction
- Hierarchical summary generation
- Context-aware content reduction

The integration of AI models, particularly through services like AWS Bedrock,
has enabled more sophisticated understanding of content relationships and
semantic structures.

#### Entity Recognition Systems

Modern entity recognition systems, exemplified by Amazon Comprehend, support
identification of various entity types:

- Technical terminology
- Named entities (people, organizations)
- Temporal references
- Quantitative information

These systems increasingly support domain-specific entity recognition, though
maintaining accuracy across different technical domains remains challenging.

The field continues to evolve in handling specialized content types and
maintaining contextual relationships across larger text spans. Integration with
AI models has enabled more sophisticated entity relationship mapping, though
challenges remain in handling domain-specific terminology and complex technical
content.

## 3. System Architecture

### 3.1 Overview

Distiller represents a modern approach to audio content analysis through a
sophisticated serverless architecture. Rather than relying on monolithic
processing, the system breaks down complex audio analysis into discrete,
manageable operations orchestrated through AWS Step Functions. This enables
parallel processing capabilities and fine-grained control over system resources
and error handling.

Each pipeline stage operates as an independent, scalable component, progressing
through input validation, audio transcription, semantic text chunking, parallel
analysis processing, and result synthesis. This design allows for isolated
maintenance and optimization of each component.

What sets Distiller apart is its data flow architecture. The system combines
direct service integrations with intermediate storage in S3, creating an
asynchronous processing model that ensures resilient state management. This
approach maintains loose coupling between components while ensuring strong
consistency through centralized workflow orchestration.

### 3.2 Core Components

#### Step Functions Workflow Engine

The heart of Distiller's orchestration lies in its AWS Step Functions
implementation. This state machine serves as the central nervous system,
coordinating all processing activities with sophisticated control flow. The
workflow engine doesn't just manage basic task execution—it handles complex
scenarios including parallel processing streams, error recovery with
configurable retry policies, and state transitions that adapt to processing
conditions.

The state machine's design incorporates branching logic to handle various
processing scenarios, particularly important when dealing with varying audio
content types and lengths. For asynchronous operations, the system implements
strategic wait states, ensuring smooth coordination between components while
maintaining processing efficiency.

#### Lambda Function Processors

The system's processing capabilities are implemented through three specialized
Lambda functions, each designed for optimal performance in its specific role:

The Extract Transcript Processor focuses on the crucial first stage of content
preparation. Written in Rust for performance, it handles the intricate process
of semantic text chunking, transforming raw transcripts into optimally sized,
contextually meaningful segments. This processor's design ensures efficient
preprocessing while maintaining semantic integrity.

The Reduce Summary Chunks Processor tackles the complex task of synthesis. It
combines and analyzes processed chunks, managing the recursive aggregation of
summaries while ensuring balanced representation of the original content. This
processor is particularly important for maintaining coherence across longer
audio content.

The Compile Text Analysis Processor serves as the final integration point,
bringing together multiple analysis streams into cohesive, well-formatted
reports. It handles not just the combination of results but also ensures proper
metadata management and formatting consistency.

These functions share common design principles: robust error handling,
comprehensive logging, and stateless operation to enable horizontal scaling.

#### Storage and Queueing Systems

Distiller's storage architecture centers on Amazon S3, which serves multiple
critical roles in the system. Beyond simple file storage, S3 acts as a
persistent state manager across processing stages, handling everything from
initial audio file ingestion to intermediate processing artifacts and final
analysis outputs. This design choice enables reliable state management while
maintaining system scalability.

### 3.3 Processing Pipeline

#### Audio Transcription Process

The pipeline begins with a sophisticated audio transcription process using AWS
Transcribe. The system is configured to handle complex audio scenarios,
supporting speaker diarization for up to 10 distinct speakers, custom vocabulary
handling for domain-specific terminology, and comprehensive language support.
The transcription process includes automatic formatting and punctuation,
ensuring high-quality input for subsequent analysis stages.

#### Semantic Chunking System

The semantic chunking system represents one of Distiller's key innovations. It
implements a sophisticated text segmentation strategy that considers multiple
factors simultaneously. The system detects natural language boundaries, speaker
transitions, and semantic completeness while optimizing chunk sizes within the
4,500-4,900 character range. This careful balancing act ensures optimal
processing downstream while maintaining contextual integrity.

#### Analysis Orchestration

Distiller's analysis phase implements parallel processing streams that maximize
throughput without sacrificing accuracy. The AI-powered analysis stream,
utilizing AWS Bedrock, handles complex tasks like content summarization and
topic extraction. Simultaneously, the natural language processing stream,
powered by Amazon Comprehend, performs entity recognition and sentiment
analysis. These parallel streams operate independently while maintaining
processing efficiency.

#### Result Compilation

The final stage brings together all analysis streams through a carefully
designed synthesis process. This multi-step approach ensures that no insights
are lost during integration. The system aggregates parallel analysis results,
incorporates NLP insights, and generates comprehensive reports with consistent
formatting and complete metadata annotation.

Throughout the pipeline, each stage maintains clear interfaces and robust error
handling. This design enables independent scaling and maintenance while ensuring
reliable end-to-end processing. The result is a system that can handle varying
workloads while maintaining processing accuracy and efficiency.

## 4. Technical Details

### 4.1 Semantic Double-Pass Merging

The semantic chunking system addresses one of the fundamental challenges in
processing long-form audio content: maintaining context across segment
boundaries while optimizing for downstream processing requirements.

As illustrated in Figure 1, the system employs a two-pass approach to text
segmentation. The first pass detects semantic boundaries and creates initial
chunks while respecting natural language structures, ensuring related concepts
remain together.

![Semantic Double-Pass Merging Process.](./assets/semantic_double_pass_merging.svg)

The system employs a sophisticated two-pass approach to text segmentation. In
the first pass, the system performs initial segmentation based on semantic
boundaries, ensuring that natural language structures like sentences and
paragraphs remain intact. This is accomplished through the text-splitter
library, which has been carefully tuned to produce chunks within a specific
character range (4,500 to 4,900 characters). This range was chosen specifically
to balance multiple competing concerns: staying within API token limits,
maintaining complete semantic units, and optimizing for processing efficiency.

The second pass involves a smart merging process that examines adjacent segments
for contextual relationships. This process ensures that related content isn't
artificially separated while still maintaining optimal chunk sizes for
downstream processing. The system prioritizes semantic completeness over strict
adherence to size limits, though it generally keeps chunks under 5,000
characters to ensure compatibility with various API limitations.

### 4.2 Recursive Analysis Architecture

Distiller's recursive analysis architecture solves a critical challenge in
processing long-form audio content: maintaining balanced representation across
the entire transcript while preserving important details. This is particularly
important for longer content where traditional single-pass approaches often show
bias toward either the beginning or end of the content.

As shown in Figure 2, the analysis process begins with multiple input chunks
that undergo progressive summarization. The first level summaries, depicted in
the middle section of Figure 2, combine related content while maintaining key
information. These intermediate summaries are then further synthesized into a
top-level summary that captures the essential points from all input chunks,
ensuring balanced representation of the original content.

![Recursive Summarization Process.](./assets/recursive_summarization.svg)

The bottom-up analysis phase begins with parallel processing of individual
chunks. Each chunk undergoes multiple types of analysis:

- Detailed summarization using Claude via Bedrock
- Topic extraction for key themes and concepts
- Entity recognition for important names, places, and terms
- Sentiment analysis to gauge emotional content and tone

The top-down synthesis phase then combines these individual analyses in a
systematic way. The system implements a hierarchical combination approach where
summaries are merged with careful attention to maintaining balanced
representation of the original content. Topics are aggregated and re-analyzed to
identify overarching themes while preserving important sub-themes from
individual sections.

### 4.3 Integration Components

The integration layer of Distiller orchestrates communication between multiple
AWS services while maintaining robust error handling and efficient resource
utilization. The system integrates with AWS Transcribe for audio processing,
Amazon Comprehend for natural language processing, and Bedrock for AI-powered
analysis.

The error handling system operates at multiple levels throughout the pipeline.
At the workflow level, the Step Functions state machine implements comprehensive
error checking and recovery mechanisms, while each Lambda function includes
configurable retry policies for different types of failures. This multi-layered
approach ensures robust operation even when dealing with transient service
issues or resource constraints.

Resource management is carefully optimized throughout the system. The Lambda
functions are implemented in Rust, chosen for its memory safety and performance
characteristics. The build configuration is optimized for minimal binary size
and maximum performance, including link-time optimization and efficient error
handling patterns. This results in fast cold starts and efficient resource
utilization during processing.

The system uses S3 for intermediate state storage, implementing efficient
streaming patterns for data transfer to avoid excessive memory usage. All
service interactions are designed to be idempotent where possible, ensuring
reliable operation even in the face of retries or parallel execution.

Throughout the implementation, the focus remains on maintaining reliability and
scalability while keeping the system modular and maintainable. Each component
has clear responsibilities and interfaces, making it easier to modify or enhance
individual parts of the system without affecting the whole.

## 5. Evaluation

### 5.1 System Performance Analysis

Distiller's capabilities have been evaluated through its application to complex
technical content analysis, specifically demonstrated through the processing of
extended academic discussions on genetic engineering and biotechnology. This
evaluation focuses on the system's performance across multiple dimensions of
content analysis and processing.

#### Processing Capabilities

The system demonstrates robust handling of technical content through several key
mechanisms:

The semantic chunking system successfully maintains context across complex
technical discussions while dividing content into processable segments. This is
particularly evident in the preservation of technical terminology and conceptual
relationships throughout the analysis pipeline.

The parallel analysis streams work in concert to provide comprehensive content
understanding. Natural language processing captures technical entities and
relationships, while AI-powered analysis extracts broader themes and conceptual
frameworks. This dual-stream approach enables both granular and high-level
content analysis.

### 5.2 Case Study: CRISPR Technology Discussion

A detailed examination of the system's performance is available through its
analysis of an extended academic discussion on CRISPR technology and genetic
engineering. This case study demonstrates the system's capabilities in
processing complex technical content while maintaining semantic coherence.

#### Content Analysis Performance

The system successfully identified and categorized main discussion themes
including:

- Technical applications and limitations of CRISPR technology
- Ethical considerations in human germline editing
- Regulatory frameworks and medical applications
- Societal and religious perspectives on genetic engineering

#### Entity Recognition

The system demonstrated precise recognition of domain-specific entities:

Technical Terms:

- CRISPR technology
- Beta thalassemia
- Sickle cell anemia

Organizations and Institutions:

- FDA
- Research institutions
- Regulatory bodies

Quantitative Information:

- Statistical references
- Technical measurements
- Temporal markers

#### Sentiment Analysis

The system provided nuanced sentiment analysis across the technical discussion:

- Neutral: 56.9% (primarily in technical explanations)
- Positive: 31.0% (associated with medical advances)
- Negative: 8.6% (concerning ethical challenges)
- Mixed: 3.6% (complex discussions)

This distribution accurately reflects the academic nature of the discourse,
appropriately capturing the predominantly neutral tone of technical discussion
while recognizing emotional elements in ethical considerations.

### 5.3 Technical Performance Characteristics

The implementation demonstrates these key operational characteristics:

#### Content Processing

- Maintains consistent context across extended technical discussions
- Preserves specialized terminology and technical relationships
- Handles multi-topic content with appropriate segmentation
- Supports complex content organization with hierarchical analysis

#### Analysis Integration

- Successfully merges parallel analysis streams
- Maintains consistency across multiple processing stages
- Produces structured, hierarchical output
- Preserves technical precision while enabling high-level analysis

These capabilities enable the system to process complex technical discussions
while maintaining both accuracy and semantic relationships.

The evaluation demonstrates Distiller's capability to handle sophisticated
technical content while maintaining analytical precision and contextual
understanding throughout the processing pipeline.

## 6. Conclusion

Through its implementation and demonstrated applications, Distiller establishes
itself as a versatile foundation for specialized audio processing needs.

### Key Findings

The implementation of Distiller reveals several important insights about
scalable audio content analysis. The system's core architecture demonstrates
remarkable adaptability across different content types while maintaining
consistent processing quality.

The combination of multiple analysis streams—AI-powered content analysis through
Bedrock and traditional NLP through Comprehend—provides comprehensive content
understanding that adapts well to different domains. This dual-stream approach
proves particularly valuable in specialized fields, where both semantic
understanding and precise entity recognition contribute to overall analysis
quality.

The serverless architecture establishes itself as an ideal foundation for
adaptable audio content analysis. The Step Functions-based orchestration manages
complex processing flows while providing the flexibility needed for
domain-specific adaptations. This architecture enables efficient resource
utilization while maintaining the adaptability required for specialized
processing needs.

The semantic chunking approach successfully balances processing efficiency with
content understanding across various domains. The system's ability to maintain
context across chunk boundaries while optimizing for downstream processing
demonstrates its potential as a foundation for specialized content analysis
solutions.

### Impact on Audio Processing

Distiller's implementation demonstrates the feasibility of creating a core
framework that can be readily adapted for specialized audio content analysis
needs while maintaining both technical precision and semantic understanding. The
system's success in processing complex technical discussions, particularly
evident in its handling of CRISPR-related content, showcases its potential as a
foundation for specialized technical analysis applications.

The framework's modular architecture, with its clear separation of concerns and
well-defined interfaces, provides a robust foundation for domain-specific
adaptations. This architectural approach enables organizations to build
specialized solutions while leveraging a proven core processing pipeline,
significantly reducing development time and complexity.

### Future Implications

The framework's successful implementation as an adaptable core system points to
several promising directions for specialized applications:

In the legal domain, Distiller's core capabilities can be enhanced with
specialized vocabulary and precedent recognition, enabling sophisticated
analysis of court proceedings and depositions. The system's strong contextual
understanding capabilities prove particularly valuable for maintaining semantic
relationships across complex legal arguments.

For medical applications, the framework's precise terminology handling and
context preservation capabilities can be adapted for processing clinical
discussions and medical conference content. The core architecture's ability to
maintain semantic relationships proves especially valuable when analyzing
complex medical procedures and clinical trial discussions.

In academic and research environments, Distiller's sophisticated handling of
technical terminology and conceptual relationships can be specialized for
comprehensive analysis of research presentations and conference proceedings. The
system's ability to maintain context across extended discussions proves
particularly valuable for tracking methodology discussions and experimental
results.

The parallel analysis architecture can be expanded to incorporate
domain-specific processing streams while maintaining the core system's
efficiency. This adaptability enables organizations to build specialized
solutions that leverage the framework's proven capabilities while addressing
their unique analytical needs.

The serverless implementation provides a robust foundation for specialized
deployments, enabling:

- Efficient resource utilization across different domains
- Scalable processing of varying content types
- Geographic distribution for improved performance
- Cost-effective handling of specialized processing requirements

Through its implementation and application, Distiller demonstrates that
sophisticated audio content analysis can be both comprehensive and adaptable.
The framework's success in maintaining both specialization capabilities and core
processing efficiency establishes it as a valuable foundation for organizations
requiring specialized audio content analysis solutions.

As the framework continues to evolve, its core architecture provides a robust
platform for developing specialized applications while preserving the
fundamental capabilities that enable accurate, efficient content analysis. This
combination of adaptability and analytical precision positions Distiller as a
valuable tool for organizations requiring sophisticated audio content analysis,
regardless of their specific domain or use case.
