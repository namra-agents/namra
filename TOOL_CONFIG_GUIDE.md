# Tool Configuration System Guide

## Overview

The tool configuration system allows agents to define pre-configured API tools in their YAML configuration files. This eliminates the need to repeat URL, headers, and other settings in every tool call.

## What Was Implemented

### 1. Tool Factory (`nexus-runtime/src/tool_factory.rs`)
- **ToolFactory**: Builds tool instances from agent configuration
- **ConfiguredHttpTool**: Wrapper that pre-configures HTTP tools with base settings
- Automatic registration of built-in generic tools
- Support for timeout parsing ("30s", "1000ms", "60")
- **Tests**: 3 unit tests (all passing)

### 2. Configuration Support
- HTTP tool configuration parsing from YAML
- Tool naming and registration
- Headers, URL, method, timeout configuration
- Extensible for future tool types (database, vector search, plugins)

### 3. CLI Integration
- Tool factory automatically used in `nexus run` command
- Tools loaded from agent config `tools:` section
- Built-in generic tools always available

## How It Works

### Before (Hardcoded Tools)
```rust
// CLI had to hardcode every tool
tools.insert("calculator", Arc::new(CalculatorTool::new()));
tools.insert("http", Arc::new(HttpTool::new()));
```

Agent had to specify everything in each call:
```
TOOL: http({"url": "https://api.example.com/data", "method": "GET", "headers": {"ApiKey": "secret"}})
```

### After (Configured Tools)
```yaml
# agent.yaml
tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://wttr.in
      method: GET
      headers:
        User-Agent: curl/7.68.0
      timeout: 10s
```

Agent can now call with simplified syntax:
```
TOOL: weather_api({"query": {"London": "?format=3"}})
```

## YAML Configuration Format

### HTTP Tool Configuration

```yaml
tools:
  - type: builtin.http          # Tool type
    name: my_api                 # Name to call it by
    config:
      url: https://api.example.com   # Base URL
      method: GET                    # HTTP method
      headers:                       # Optional headers
        Authorization: Bearer token
        User-Agent: MyAgent/1.0
      timeout: 30s                   # Request timeout
      retry: false                   # Retry on failure
```

### Filesystem Tool Configuration

```yaml
tools:
  # Local filesystem with sandboxing
  - type: builtin.filesystem
    name: workspace_files
    config:
      type: local
      read_only: false              # Allow write operations
      backend_type: local
      base_dir: /tmp/agent-workspace  # Sandbox root

  # S3 storage (Week 12+)
  - type: builtin.filesystem
    name: s3_storage
    config:
      type: s3
      read_only: false
      backend_type: s3
      bucket: my-bucket
      region: us-east-1
      prefix: agents/               # Optional path prefix
      credentials: ~/.aws/credentials  # Optional

  # Google Cloud Storage (Week 12+)
  - type: builtin.filesystem
    name: gcs_storage
    config:
      type: gcs
      read_only: true
      backend_type: gcs
      bucket: my-gcs-bucket
      project: my-project           # Optional
      prefix: data/                 # Optional
      credentials: ~/.gcloud/key.json  # Optional

  # Azure Blob Storage (Week 12+)
  - type: builtin.filesystem
    name: azure_storage
    config:
      type: azure
      read_only: false
      backend_type: azure
      container: my-container
      account: mystorageaccount
      prefix: files/                # Optional
      credentials: ~/.azure/credentials  # Optional

  # SFTP (Week 12+)
  - type: builtin.filesystem
    name: sftp_server
    config:
      type: sftp
      read_only: false
      backend_type: sftp
      host: sftp.example.com
      port: 22                      # Default: 22
      username: myuser
      base_path: /home/myuser       # Optional
      credentials: ~/.ssh/id_rsa    # Optional
```

**Filesystem operations:**
- `read`: Read file contents
- `write`: Write content to file
- `list`: List directory contents
- `delete`: Delete a file

**Example usage:**
```
TOOL: workspace_files({"operation": "read", "path": "data.txt"})
TOOL: workspace_files({"operation": "write", "path": "output.txt", "content": "Hello"})
TOOL: workspace_files({"operation": "list", "path": "."})
TOOL: workspace_files({"operation": "delete", "path": "temp.txt"})
```

### Multiple Tool Configurations

```yaml
tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://wttr.in
      method: GET
      timeout: 10s

  - type: builtin.http
    name: news_api
    config:
      url: https://newsapi.org/v2
      method: GET
      headers:
        X-Api-Key: your-key-here
      timeout: 15s

  - type: builtin.http
    name: post_endpoint
    config:
      url: https://api.example.com/data
      method: POST
      headers:
        Content-Type: application/json
      timeout: 20s
```

## Tool Types Supported

### Currently Implemented

#### 1. `builtin.http`
Pre-configured HTTP API endpoints
- **Config fields**: url, method, headers, timeout, retry
- **Status**: ✅ Fully implemented

#### 2. `builtin.filesystem`
Configurable filesystem with multiple backends
- **Config fields**: type (local/s3/gcs/azure/sftp), read_only, backend-specific settings
- **Backends**: Local (✅ implemented), S3/GCS/Azure/SFTP (⏳ Week 12+)
- **Operations**: read, write, list, delete
- **Status**: ✅ Local backend implemented, remote stubs added

#### 3. Built-in Generic Tools (Always Available)
- **calculator** - Arithmetic operations
- **string** - String manipulation
- **Status**: ✅ Auto-registered

### Planned for Future

#### 3. `builtin.database` - Week 12+
Database query tool
```yaml
- type: builtin.database
  name: user_db
  config:
    connection: postgresql
    connection_string: postgres://localhost/mydb
    query_type: read
    max_rows: 100
```

#### 4. `builtin.vector_search` - Week 12+
Vector similarity search
```yaml
- type: builtin.vector_search
  name: doc_search
  config:
    index: documents
    top_k: 5
    similarity_threshold: 0.8
```

#### 5. `plugin.python` - Week 7+
Python custom tools
```yaml
- type: plugin.python
  name: custom_tool
  module: my_tools
  function: process_data
  timeout: 60s
```

#### 6. `agent` - Week 13+
Agent-as-tool (sub-agents)
```yaml
- type: agent
  name: specialist
  agent_ref: specialist_agent.yaml
  timeout: 120s
```

## Example: Weather Agent

See `test-agents/weather_agent.yaml`:

```yaml
name: weather_agent
version: 1.0.0

llm:
  provider: anthropic
  model: claude-sonnet-4-5-20250929

tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://wttr.in
      method: GET
      headers:
        User-Agent: curl/7.68.0
      timeout: 10s

system_prompt: |
  You have access to weather_api tool.
  Use it like: TOOL: weather_api({"query": {"London": "?format=3"}})
```

## Example: File Manager Agent

See `test-agents/file_manager_agent.yaml`:

```yaml
name: file_manager
version: 1.0.0

llm:
  provider: anthropic
  model: claude-sonnet-4-5-20250929

tools:
  - type: builtin.filesystem
    name: workspace_files
    config:
      type: local
      read_only: false
      backend_type: local
      base_dir: /tmp/agent-workspace

system_prompt: |
  You have access to workspace_files tool with operations:
  - read: TOOL: workspace_files({"operation": "read", "path": "file.txt"})
  - write: TOOL: workspace_files({"operation": "write", "path": "file.txt", "content": "data"})
  - list: TOOL: workspace_files({"operation": "list", "path": "."})
  - delete: TOOL: workspace_files({"operation": "delete", "path": "file.txt"})
```

## How to Use Pre-configured Tools

### In Agent System Prompt

```yaml
system_prompt: |
  You have access to the following tools:

  1. **weather_api** - Get weather for any location
     Syntax: TOOL: weather_api({"query": {"<city>": "?format=3"}})
     Example: TOOL: weather_api({"query": {"London": "?format=3"}})

  2. **calculator** - Perform calculations
     Syntax: TOOL: calculator(expression)
```

### Agent Tool Call Examples

**Weather lookup:**
```
User: What's the weather in Tokyo?
Agent: TOOL: weather_api({"query": {"Tokyo": "?format=3"}})
```

**With calculation:**
```
User: What's the weather in NYC?
Agent: TOOL: weather_api({"query": {"NYC": "?format=3"}})
[Gets: "NYC: ☀️ +5°C"]
Agent: Let me convert that to Fahrenheit. TOOL: calculator(5 * 9/5 + 32)
[Gets: "41"]
Agent: ANSWER: It's sunny and 5°C (41°F) in New York City.
```

## Testing

### Test the Weather Agent

```bash
cargo run --bin nexus -- run test-agents/weather_agent.yaml \
  --input "What's the weather in London?"
```

**Expected behavior:**
1. Agent loads with `weather_api` configured tool
2. Agent calls: `TOOL: weather_api({"query": {"London": "?format=3"}})`
3. Tool executes HTTP GET to `https://wttr.in/London?format=3`
4. Agent receives weather data
5. Agent responds with weather information

### Test Generic Tools (Always Work)

```bash
cargo run --bin nexus -- run test-agents/flight_agent.yaml \
  --input "What is 42 * 137?"
```

The calculator tool is always available, even if not in `tools:` section.

## Architecture

```
Agent YAML Config
    │
    ├── tools: []
    │   └── HTTP tool configs
    │
    ↓
ToolFactory
    │
    ├── Parses tool configs
    ├── Creates tool instances
    ├── Registers generic tools
    │
    ↓
Tool Registry (HashMap)
    │
    ├── "calculator" → CalculatorTool
    ├── "string" → StringTool
    ├── "flight_prices" → MockFlightTool
    ├── "weather_api" → ConfiguredHttpTool
    │
    ↓
AgentExecutor
    │
    └── Makes tools available to ReAct strategy
```

## Benefits

### 1. **Cleaner Tool Calls**
Before:
```
TOOL: http({"url": "https://api.example.com/data", "method": "GET", "headers": {"Auth": "token"}})
```

After:
```
TOOL: api_data({})
```

### 2. **Security**
- API keys stored in config, not in prompts
- Credentials separate from agent logic
- Headers pre-configured and hidden from agent

### 3. **Reusability**
- Define tool once, use many times
- Share tool configs across agents
- Standard library of configured tools

### 4. **Maintainability**
- Change API endpoint in one place
- Update headers without changing prompts
- Version control for tool configurations

## Implementation Details

### ConfiguredHttpTool

Wraps the generic HTTP tool with pre-configured settings:

```rust
pub struct ConfiguredHttpTool {
    name: String,           // Tool name (e.g., "weather_api")
    base_url: String,       // Pre-configured URL
    method: String,         // Pre-configured method
    headers: HashMap,       // Pre-configured headers
    timeout: Duration,      // Pre-configured timeout
    http_tool: HttpTool,    // Underlying generic tool
}
```

When called, it:
1. Merges config defaults with call-time parameters
2. Adds query parameters to URL if provided
3. Executes using underlying HTTP tool
4. Returns result

### Extensibility

To add a new tool type:

1. Define config struct in `nexus-config/src/agent.rs`:
   ```rust
   pub struct MyToolConfig {
       pub setting1: String,
       pub setting2: u32,
   }
   ```

2. Add variant to `ToolConfig` enum:
   ```rust
   pub enum ToolConfig {
       MyTool { name: String, config: MyToolConfig },
   }
   ```

3. Implement in `ToolFactory::build_tool_from_config()`:
   ```rust
   ToolConfig::MyTool { name, config } => {
       let tool = self.build_my_tool(config)?;
       Ok((name.clone(), Arc::new(tool)))
   }
   ```

## Statistics

- **New code**: ~800 lines total
  - tool_factory.rs: ~300 lines
  - filesystem backends: ~500 lines (local + remote stubs)
- **Tests**: 61 total tests across workspace (all passing)
  - Filesystem backend tests: 8 tests
  - Tool factory tests: 3 tests
- **Example agents**: 2 (weather_agent.yaml, file_manager_agent.yaml)
- **Supported tool types**: 2 (HTTP, Filesystem with local backend), 4 planned

## Next Steps

1. **Test with real APIs** - Try GitHub API, OpenWeather, etc.
2. **Add auth support** - Bearer tokens, API keys in config
3. **Database tools** - SQL query execution (Week 12)
4. **Vector search** - Semantic search tools (Week 12)
5. **Python plugins** - Custom tool server (Week 7)

## Conclusion

The tool configuration system provides:
- ✅ Pre-configured API endpoints in YAML
- ✅ Simplified tool calling syntax
- ✅ Security through credential separation
- ✅ Extensible architecture for future tool types
- ✅ Backward compatible with generic tools

Agents can now have their own configured tools while still accessing all built-in generic tools!
