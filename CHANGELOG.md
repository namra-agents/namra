# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
