# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Run History & Storage** (`namra-storage` crate):
  - SQLite-backed storage of agent execution history
  - Automatic run persistence with full execution details
  - `namra runs list` - List recent runs with filtering
  - `namra runs show <id>` - Show detailed run information
  - `namra runs export` - Export to CSV, JSON, Excel formats
  - `namra runs stats` - Execution statistics and analytics
  - `namra runs delete` - Delete old runs
  - Tool call and reasoning step tracking

- **OpenTelemetry Observability** (`namra-middleware` crate):
  - OpenTelemetry integration with distributed tracing
  - Multiple exporter support:
    - **Jaeger** - OTLP gRPC exporter for general distributed tracing
    - **Phoenix** - OTLP HTTP exporter for LLM-specific observability
    - **OTLP** - Generic OTLP gRPC/HTTP exporters
    - **Stdout** - Console output for debugging
  - Span instrumentation for agent runs, LLM requests, and tool executions
  - Content capture (opt-in) for prompts, responses, and tool I/O
  - Automatic content truncation for OTEL limits (4KB)
  - Comprehensive span attributes:
    - LLM: provider, model, tokens (input/output), cost, prompts, responses
    - Tools: name, success status, duration, input, output
    - Agent: name, version, iterations, stop reason
  - Environment variable configuration support
  - Configurable sampling rates and trace options

- **Test Configurations**:
  - `test_observability.yaml` - Jaeger observability test agent
  - `test_phoenix.yaml` - Phoenix observability test agent
  - Structured test configs with clear documentation

### Changed

- Enhanced agent configurations to support observability middleware
- Updated example agents with observability examples
- Improved CLI output for observability status

## [0.1.0] - 2026-01-29

### Added

- **Agent Runtime**: ReAct (Reasoning and Acting) strategy for agent execution
- **LLM Support**: Anthropic Claude integration with streaming
- **Built-in Tools**:
  - Calculator - arithmetic operations
  - String - text manipulation
  - HTTP - configurable API calls
  - Filesystem - file operations with sandboxing and backend abstraction
- **Tool Configuration**: Pre-configure HTTP and filesystem tools via YAML
- **CLI**: `namra run` command to execute agents
- **Configuration**: YAML-based agent configuration with validation
- **Execution Tracking**: Token usage, cost calculation, iteration limits
- **Reasoning Display**: Show agent's step-by-step thinking process
- **Cross-platform**: Prebuilt binaries for macOS (Intel/ARM), Linux, Windows

### Security

- Filesystem sandboxing with configurable base directory
- Path traversal prevention
- Read-only mode for filesystem tool

[Unreleased]: https://github.com/namra-agents/namra/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/namra-agents/namra/releases/tag/v0.1.0
