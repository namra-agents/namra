# Namra Implementation Roadmap

This roadmap outlines the 20-week implementation plan for the Namra enterprise agent framework.

## Vision

**Namra** is a Rust-powered, config-driven agent framework that combines:
- **Performance**: 10-100× faster than Python frameworks
- **Simplicity**: YAML configs (no code for 80% of use cases)
- **Enterprise-Ready**: Observability, security, governance built-in
- **Flexibility**: Python SDK for custom tools (20% of cases)

## Current Status (Week 1 - COMPLETED ✅)

### What's Done
- ✅ Rust workspace with 9 crates
- ✅ `namra-config`: Full YAML/TOML parsing and validation
- ✅ `namra-cli`: Working CLI with `init` and `validate` commands
- ✅ Example configurations
- ✅ Documentation (README, this ROADMAP)

### What Works Now
```bash
# Initialize new project
namra init my-agent

# Validate configs
namra validate agents/*.yaml
```

---

## Phase 1: Core Foundation (Weeks 1-4) - MVP

### Week 1: Project Setup & Config Parsing ✅ DONE
**Status**: Completed

**Deliverables**:
- [x] Rust workspace with all crate directories
- [x] Configuration parsing (YAML/TOML)
- [x] CLI tool (`namra init`, `namra validate`)
- [x] Example configurations

### Week 2: LLM Adapters 🚧 NEXT
**Goal**: Implement LLM provider integrations

**Tasks**:
- [ ] Design LLM adapter trait in `namra-llm`
  - Unified async interface
  - Streaming support
  - Error types
- [ ] Implement Anthropic adapter
  - Messages API integration
  - Streaming with SSE
  - Tool use support
  - Token counting
- [ ] Implement OpenAI adapter
  - Chat completions API
  - Streaming
  - Function calling
- [ ] Unit tests with mocked responses

**Files to Create**:
```
namra-core/namra-llm/
├── src/
│   ├── lib.rs              # Public API
│   ├── types.rs            # Common types (LLMRequest, LLMResponse)
│   ├── adapter.rs          # LLMAdapter trait
│   ├── anthropic.rs        # Anthropic implementation
│   ├── openai.rs           # OpenAI implementation
│   └── error.rs            # Error types
└── tests/
    └── integration_tests.rs
```

**Deliverable**: Can call Anthropic/OpenAI APIs from Rust

### Week 3: Built-in Tools
**Goal**: Implement core built-in tools

**Tasks**:
- [ ] Design tool system in `namra-tools`
  - Tool trait
  - Input/output validation
  - Timeout handling
- [ ] Implement HTTP tool
  - GET/POST/PUT/DELETE
  - Headers and auth
  - Response parsing
- [ ] Implement basic tools
  - File system operations
  - Simple calculations
- [ ] Tool execution sandbox
  - Resource limits
  - Timeout enforcement

**Files to Create**:
```
namra-core/namra-tools/
├── src/
│   ├── lib.rs              # Public API
│   ├── tool.rs             # Tool trait
│   ├── http.rs             # HTTP tool
│   ├── filesystem.rs       # File operations
│   └── builtin.rs          # Calculator, etc.
└── tests/
```

**Deliverable**: Working built-in tools

### Week 4: Agent Engine (ReAct)
**Goal**: Implement basic agent execution

**Tasks**:
- [ ] Design agent engine in `namra-runtime`
  - Execution context
  - Message handling
  - Tool calling loop
- [ ] Implement ReAct strategy
  - Thought → Action → Observation loop
  - Max iterations limit
  - Stop conditions
- [ ] In-memory message storage
  - Conversation history
  - Simple retrieval
- [ ] Basic error handling and recovery

**Files to Create**:
```
namra-core/namra-runtime/
├── src/
│   ├── lib.rs              # Public API
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── executor.rs     # Agent executor
│   │   ├── context.rs      # Execution context
│   │   └── strategies/
│   │       └── react.rs    # ReAct strategy
│   └── message.rs          # Message types
└── tests/
```

**MVP Deliverable (End of Week 4)**:
```python
# Can run a simple ReAct agent end-to-end
namra run agent.yaml --input "What's 25 * 47?"
```

---

## Phase 2: API & Python SDK (Weeks 5-8)

### Week 5: gRPC/HTTP Server
**Goal**: Expose runtime via gRPC and REST

**Tasks**:
- [ ] Design protocol buffers in `namra-api`
- [ ] Implement gRPC server
  - ExecuteAgent endpoint
  - Streaming responses
- [ ] Implement HTTP REST API (Axum)
  - JSON request/response
  - SSE for streaming
- [ ] Authentication & authorization

**Deliverable**: Runtime accessible via API

### Week 6: Python SDK Foundation
**Goal**: Create Python client library

**Tasks**:
- [ ] Generate Python code from protobuf
- [ ] Implement `NamraClient` class
- [ ] Type definitions using Pydantic
- [ ] Unit tests with mocked gRPC

**Deliverable**: Python SDK can execute agents

### Week 7: Python Tool System
**Goal**: Enable custom Python tools

**Tasks**:
- [ ] Implement `@tool` decorator
- [ ] Implement tool server (FastAPI)
- [ ] Rust side: Python plugin bridge
- [ ] Integration tests

**Deliverable**: Can register and execute Python tools

### Week 8: CLI Enhancement
**Goal**: Rich CLI experience

**Tasks**:
- [ ] Implement `namra run`
- [ ] Implement `namra logs`
- [ ] Implement `namra metrics`
- [ ] Shell completions

**Deliverable**: Full-featured CLI

---

## Phase 3: Observability & Resilience (Weeks 9-12)

### Week 9: OpenTelemetry Integration
**Goal**: Full distributed tracing

**Tasks**:
- [ ] Implement observability middleware
- [ ] Trace every operation
- [ ] Metrics collection (latency, tokens, cost)
- [ ] Structured logging

**Deliverable**: Full observability with Jaeger

### Week 10: Security Middleware
**Goal**: Enterprise-grade security

**Tasks**:
- [ ] Input validation
- [ ] Rate limiting
- [ ] Secrets management (Vault)
- [ ] Prompt injection detection

**Deliverable**: Secure runtime

### Week 11: Resilience Middleware
**Goal**: Production-grade resilience

**Tasks**:
- [ ] Circuit breaker
- [ ] Retry logic with exponential backoff
- [ ] Timeout management
- [ ] Fallback strategies

**Deliverable**: Resilient agent execution

### Week 12: Memory Systems
**Goal**: Persistent memory stores

**Tasks**:
- [ ] Redis memory implementation
- [ ] PostgreSQL memory implementation
- [ ] Message summarization
- [ ] Integration tests

**Deliverable**: Multiple memory backends

---

## Phase 4: Advanced Features (Weeks 13-16)

### Week 13: Workflow Engine
**Goal**: DAG-based orchestration

**Tasks**:
- [ ] Design workflow executor
- [ ] Implement conditional routing
- [ ] Parallel execution (fan-out/fan-in)
- [ ] Checkpoint and resume

**Deliverable**: Working workflow engine

### Week 14: Multi-Agent System
**Goal**: Agent collaboration

**Tasks**:
- [ ] Message bus implementation (Redis pub/sub)
- [ ] Agent registry
- [ ] Hierarchical coordination
- [ ] Consensus mechanisms

**Deliverable**: Multi-agent coordination

### Week 15: Governance Middleware
**Goal**: Policy enforcement and compliance

**Tasks**:
- [ ] Policy engine
- [ ] PII detection
- [ ] Cost tracking and limits
- [ ] Audit logging

**Deliverable**: Governance controls

### Week 16: Caching Layer
**Goal**: Intelligent response caching

**Tasks**:
- [ ] Exact match caching
- [ ] Semantic caching (embedding-based)
- [ ] Cache warming
- [ ] Metrics and monitoring

**Deliverable**: Intelligent caching

---

## Phase 5: Enterprise & Polish (Weeks 17-20)

### Week 17: Multi-Tenancy
**Goal**: Tenant isolation and resource management

**Tasks**:
- [ ] Tenant context propagation
- [ ] Resource quotas
- [ ] Data isolation
- [ ] Billing and metering

**Deliverable**: Multi-tenant support

### Week 18: Deployment Tools
**Goal**: Production deployment

**Tasks**:
- [ ] Docker images
- [ ] Kubernetes manifests
- [ ] Helm chart
- [ ] Monitoring setup (Prometheus, Grafana)

**Deliverable**: Production-ready deployment

### Week 19: Documentation & Examples
**Goal**: Excellent developer experience

**Tasks**:
- [ ] Comprehensive documentation
- [ ] Example agents (code reviewer, support, etc.)
- [ ] Example workflows
- [ ] Tutorial videos

**Deliverable**: Complete documentation

### Week 20: Testing & Benchmarking
**Goal**: Production quality

**Tasks**:
- [ ] Comprehensive test suite (>80% coverage)
- [ ] Performance benchmarks
- [ ] Comparison benchmarks vs competitors
- [ ] Load testing
- [ ] Security audit

**Deliverable**: 1.0 Release Ready

---

## Configuration Management & UX Improvements (Future Discussion)

**Note**: Discuss and prioritize these features later

### Config Change Experience
- [ ] **Config Diff Command**: `namra diff old.yaml new.yaml`
  - Show what changed between config versions
  - Highlight impact of changes

- [ ] **Dry Run Mode**: `namra run agent.yaml --dry-run`
  - Validate config without calling LLM
  - Show estimated cost and token usage
  - Preview execution plan

- [ ] **CLI Config Override**: `namra run agent.yaml --override llm.temperature=0.0`
  - Quick parameter testing without editing files
  - Multiple overrides: `--override llm.temperature=0.0 --override llm.max_tokens=100`
  - Useful for A/B testing different configurations

- [ ] **Watch Mode**: `namra run agent.yaml --watch`
  - Auto-reload and rerun when config file changes
  - Great for development/experimentation
  - Show diff of what changed

- [ ] **Hot Reload for Long-Running Agents**
  ```yaml
  hot_reload:
    enabled: true
    watch_files: ["agents/*.yaml", "tools/*.py"]
  ```

### Agent Lifecycle Management

#### Current State (Week 2)
- ✅ CLI mode only (synchronous execution)
- ✅ Stop with `Ctrl+C` (SIGINT)
- ❌ No graceful shutdown
- ❌ No daemon mode

#### Planned Improvements

**Week 8-9: Graceful Shutdown**
- [ ] Handle SIGINT/SIGTERM gracefully
- [ ] Cleanup in-progress tool calls
- [ ] Save conversation state before exit
- [ ] Configurable shutdown timeout

**Week 18: Daemon Mode & Process Management**
- [ ] **Run as Daemon**: `namra run agent.yaml --daemon`
  - Background execution
  - PID file management
  - Automatic restart on failure

- [ ] **Stop Commands**:
  ```bash
  namra stop <agent-name>              # Stop specific agent
  namra stop --all                     # Stop all agents
  namra restart <agent-name>           # Restart agent
  namra kill <agent-name>              # Force kill (SIGKILL)
  ```

- [ ] **Status Management**:
  ```bash
  namra status                         # List all running agents
  namra status <agent-name>            # Detailed status
  namra logs <agent-name> --follow     # Tail logs
  namra ps                             # Process list (like docker ps)
  ```

- [ ] **Agent State Persistence**:
  - Save conversation history
  - Resume from checkpoint
  - Graceful handoff between versions

**Example Workflow (Future)**:
```bash
# Start agent as daemon
namra run agents/support-agent.yaml --daemon
# Output: Agent 'support-agent' started (PID: 12345)

# Check status
namra status support-agent
# Output:
#   Name: support-agent
#   Status: Running
#   PID: 12345
#   Uptime: 2h 34m
#   Requests: 1,247
#   Cost: $3.45

# Update config (agent auto-reloads if hot-reload enabled)
vim agents/support-agent.yaml

# Or manually restart to apply changes
namra restart support-agent
# Output: Agent 'support-agent' restarted (PID: 12389)

# Stop when done
namra stop support-agent
# Output: Agent 'support-agent' stopped gracefully
```

### Signal Handling Behavior

| Signal | Current | Planned |
|--------|---------|---------|
| **SIGINT** (Ctrl+C) | Immediate exit | Graceful shutdown (finish current request) |
| **SIGTERM** | Not handled | Graceful shutdown (configurable timeout) |
| **SIGKILL** | Immediate kill | Immediate kill (no cleanup) |
| **SIGHUP** | Not handled | Reload config without restart |

### Configuration Reload Strategies

```yaml
# agent.yaml (future)
lifecycle:
  shutdown:
    grace_period: 30s              # Wait for current request
    save_state: true               # Save conversation before exit

  reload:
    strategy: graceful             # Options: graceful, immediate
    on_config_change: reload       # Options: reload, restart, ignore
    preserve_memory: true          # Keep conversation history
```

## Phase 6+: Future Enhancements (Month 6+)

### Visual Builder (Weeks 21-24)
- React-based canvas UI
- Drag-drop agent configuration
- Live execution monitoring
- YAML ↔ Visual converter

**Effort**: 17 weeks
**Priority**: Medium (nice-to-have, not required)

### Additional Features
- Real-time voice agents
- Multi-modal support (images, audio, video)
- Additional LLM providers (Google, AWS Bedrock, local models)
- WebAssembly plugin support
- Multi-cluster Kubernetes deployment

---

## Success Metrics

### Technical Metrics
- **Cold Start**: <20ms (target: 10-15ms)
- **Memory**: <50MB base (target: 30-40MB)
- **Throughput**: >1000 req/s on 2 CPU cores
- **P99 Latency**: <100ms overhead (excluding LLM time)
- **Binary Size**: <10MB compressed
- **Test Coverage**: >85%

### Comparison Benchmarks

| Metric | Namra Target | Agno | OpenAI AgentKit | LangChain |
|--------|--------------|------|-----------------|-----------|
| Cold start | **15ms** | 150ms | 200ms | 500ms |
| Memory | **40MB** | 80MB | 100MB | 300MB |
| Throughput | **1200 req/s** | 300 | 250 | 100 |

### Adoption Goals
- **Month 3**: 100 GitHub stars, 10 production users
- **Month 6**: 1K GitHub stars, 50 production users
- **Month 12**: 5K GitHub stars, 200+ production users

---

## Critical Files Reference

### Phase 1 (MVP - Weeks 1-4)
1. `namra-core/namra-config/src/parser.rs` - Config parsing ✅
2. `namra-core/namra-llm/src/anthropic.rs` - First LLM adapter 🚧 NEXT
3. `namra-core/namra-tools/src/http.rs` - First built-in tool
4. `namra-core/namra-runtime/src/agent/executor.rs` - Core execution loop
5. `namra-core/namra-cli/src/commands/run.rs` - Run command

### Phase 2 (API & SDK - Weeks 5-8)
6. `namra-core/namra-api/proto/namra.proto` - API contract
7. `namra-py/namra_sdk/client.py` - Python client
8. `namra-core/namra-middleware/src/observability.rs` - OpenTelemetry

---

## Architecture Decisions

### Why Rust?
- **Performance**: 10-100× faster than Python
- **Memory Safety**: No crashes from null pointers, race conditions
- **Small Footprint**: <50MB vs 200-300MB for Python
- **Type Safety**: Catch errors at compile time

### Why Config-First (YAML)?
- **Lower Barrier**: Non-programmers can create agents
- **GitOps-Friendly**: Version control, code review, CI/CD
- **Declarative**: What you want, not how to do it
- **Infrastructure-as-Code**: Like Kubernetes

### Why Python SDK (Optional)?
- **Familiarity**: Most ML/AI developers know Python
- **Flexibility**: For the 20% of custom tool cases
- **Ecosystem**: Leverage Python libraries when needed

### Why Not Kubernetes-Dependent?
- **Broader Adoption**: Works on laptops, VMs, Docker, K8s
- **Simpler Development**: No K8s needed for testing
- **Flexibility**: Deploy anywhere

---

## Quick Reference Commands

```bash
# Development
cargo build                       # Build all crates
cargo test                        # Run tests
cargo build --release            # Optimized build

# CLI Usage
namra init my-agent              # Create new project
namra validate agent.yaml        # Validate config
namra run agent.yaml             # Run agent (Week 4+)
namra deploy agent.yaml          # Deploy to prod (Week 18+)

# Testing
cargo test -p namra-config       # Test specific crate
cargo bench                       # Run benchmarks
```

---

## Next Session Checklist

When continuing development, start here:

### Week 2: LLM Adapters (Current Priority)

1. **Create LLM types** (`namra-llm/src/types.rs`):
   - `LLMRequest`, `LLMResponse`
   - `Message`, `MessageRole`
   - Streaming types

2. **Define LLM trait** (`namra-llm/src/adapter.rs`):
   ```rust
   #[async_trait]
   pub trait LLMAdapter {
       async fn generate(&self, req: LLMRequest) -> Result<LLMResponse>;
       async fn stream(&self, req: LLMRequest) -> Result<LLMStream>;
   }
   ```

3. **Implement Anthropic** (`namra-llm/src/anthropic.rs`):
   - Messages API integration
   - Streaming via SSE
   - Error handling
   - Rate limiting

4. **Add integration tests**:
   - Mock responses
   - Real API tests (with env var)

5. **Update CLI** to use LLM:
   - Add `namra run` command
   - Execute simple prompts

---

## Resources

### Documentation
- [Full Architecture Plan](/.claude/plans/drifting-wandering-eich.md)
- [README](README.md)
- [This Roadmap](ROADMAP.md)

### External References
- [Anthropic API Docs](https://docs.anthropic.com/claude/reference)
- [OpenAI API Docs](https://platform.openai.com/docs/api-reference)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Last Updated**: Week 1 completed
**Next Milestone**: Week 2 - LLM Adapters
**Target**: MVP by end of Week 4
