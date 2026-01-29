# Nexus Core Tools

## Overview

The Nexus platform includes 4 basic generic tools that are always available to all agents. These tools provide fundamental capabilities without being domain-specific.

## Philosophy

**Core tools should be:**
- ✅ Generic and widely applicable
- ✅ Simple and focused
- ✅ No external dependencies (except HTTP client)
- ✅ Safe and sandboxed where necessary

**Not core tools:**
- ❌ Domain-specific (e.g., flight prices, weather)
- ❌ Require external APIs/accounts
- ❌ Complex business logic

## The 4 Core Tools

### 1. Calculator Tool
**Purpose:** Arithmetic operations
**Operations:** +, -, *, /
**Availability:** ✅ Always auto-registered
**Use cases:**
- Mathematical calculations
- Price comparisons
- Unit conversions
- Simple formulas

**Example:**
```
TOOL: calculator(123 * 456)
Result: "123 * 456 = 56088"
```

**Code:** `nexus-tools/src/builtin.rs` (CalculatorTool)
**Tests:** 7 unit tests

---

### 2. String Tool
**Purpose:** Text manipulation
**Operations:** uppercase, lowercase, reverse, length, trim, replace
**Availability:** ✅ Always auto-registered
**Use cases:**
- Text formatting
- String transformations
- Text analysis
- Data cleaning

**Example:**
```
TOOL: string({"operation": "uppercase", "text": "hello world"})
Result: "HELLO WORLD"
```

**Code:** `nexus-tools/src/builtin.rs` (StringTool)
**Tests:** 9 unit tests

---

### 3. HTTP Tool
**Purpose:** Make HTTP requests
**Methods:** GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
**Availability:** ⚙️ Configurable (can be pre-configured per agent)
**Use cases:**
- API calls
- Web scraping
- Webhook triggers
- Data fetching

**Generic example:**
```
TOOL: http({"url": "https://api.example.com/data", "method": "GET"})
```

**Pre-configured example:**
```yaml
# In agent config
tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://wttr.in
      method: GET
      timeout: 10s

# In agent call
TOOL: weather_api({"path": "/London", "query": {"format": "3"}})
```

**Features:**
- Custom headers
- Request body (JSON/text)
- Configurable timeout
- Response metadata
- Pre-configuration for specific APIs

**Code:** `nexus-tools/src/http.rs`
**Tests:** 11 unit tests (including real HTTP calls)

---

### 4. Filesystem Tool
**Purpose:** File operations
**Operations:** read, write, list, delete
**Availability:** ⚙️ Configurable (supports local and remote backends)
**Use cases:**
- Reading config files
- Writing logs
- Directory listing
- File management

**Configuration required:**
```yaml
# In agent config
tools:
  - type: builtin.filesystem
    name: workspace_files
    config:
      type: local
      read_only: false
      backend_type: local
      base_dir: /tmp/agent-workspace

# In agent call
TOOL: workspace_files({"operation": "read", "path": "data.txt"})
```

**Features:**
- Multiple backends (local, S3, GCS, Azure, SFTP)
- Sandboxing with base directory
- Path validation and security
- Read-only mode support
- Local backend fully implemented
- Remote backends (Week 12+)

**Code:** `nexus-tools/src/filesystem/` (modular backend architecture)
**Tests:** 8 unit tests (local backend)

---

## Usage

### Automatic vs Configured Tools

**Always Auto-registered (2 tools):**
- `calculator` - Arithmetic operations
- `string` - Text manipulation

```rust
// In ToolFactory::build_tools()
tools.insert("calculator", Arc::new(CalculatorTool::new()));
tools.insert("string", Arc::new(StringTool::new()));
```

**Configurable Tools (2 tools):**
- `http` - Pre-configured with specific API settings
- `filesystem` - Configured with backend (local/S3/GCS/Azure/SFTP) and sandboxing

These must be defined in the agent's `tools:` section in YAML config.

### In Agent System Prompts

```yaml
system_prompt: |
  You have access to these tools:

  1. calculator - Perform arithmetic: calculator(expression)
  2. string - Text manipulation: string({"operation": "op", "text": "..."})
  3. http - Make HTTP requests: http({"url": "...", "method": "GET"})
  4. filesystem - File operations: filesystem({"operation": "read", "path": "..."})
```

## Tool Statistics

| Tool | LOC | Tests | Dependencies |
|------|-----|-------|--------------|
| Calculator | ~200 | 7 | None |
| String | ~200 | 9 | None |
| HTTP | ~300 | 11 | reqwest |
| Filesystem | ~400 | 9 | tokio::fs, tempfile (test) |
| **Total** | **~1,100** | **36** | - |

## Why These 4?

### Calculator
**Rationale:** Essential for any quantitative reasoning. Pure function, no side effects.

### String
**Rationale:** Text is universal. Most agents need basic text manipulation.

### HTTP
**Rationale:** Gateway to the internet. Enables integration with any API without custom tools.

### Filesystem
**Rationale:** Local data access. Critical for reading configs, writing logs, managing files.

## What's NOT Included

### Removed Tools
- **MockFlightTool** - Domain-specific, testing only
- **DatabaseTool** - Not yet implemented (Week 12+)
- **VectorSearchTool** - Not yet implemented (Week 12+)

### Why Not More?

**Keep it simple:**
- Fewer tools = easier to learn
- Generic > Specific
- Extensible via HTTP tool

**Examples of tools that should NOT be core:**
- Weather API - Use HTTP tool
- Flight prices - Use HTTP tool
- Email sending - Use HTTP tool
- Payment processing - Use HTTP tool
- Social media - Use HTTP tool

**HTTP tool is the universal adapter!**

## How to Add Domain-Specific Tools

### Option 1: Pre-configure HTTP Tool
Pre-configure an HTTP endpoint in agent config:

```yaml
tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://api.weather.com
      headers:
        ApiKey: your-key
      timeout: 10s
```

### Option 2: Configure Filesystem Tool
Configure a filesystem backend for file operations:

```yaml
tools:
  # Local sandboxed filesystem
  - type: builtin.filesystem
    name: workspace
    config:
      type: local
      read_only: false
      backend_type: local
      base_dir: /tmp/workspace

  # S3 storage (Week 12+)
  - type: builtin.filesystem
    name: s3_data
    config:
      type: s3
      backend_type: s3
      bucket: my-bucket
      region: us-east-1
```

### Option 3: Python Plugin (Week 7+)
Create custom tools in Python:

```yaml
tools:
  - type: plugin.python
    name: custom_tool
    module: my_tools
    function: process_data
```

### Option 4: Custom Rust Tool
For high-performance needs, implement the Tool trait in Rust.

## Testing

All core tools have comprehensive tests:

```bash
# Test all core tools
cargo test -p nexus-tools --lib

# Test specific tool
cargo test -p nexus-tools calculator
cargo test -p nexus-tools string
cargo test -p nexus-tools http
cargo test -p nexus-tools filesystem
```

## Security Considerations

### Calculator
✅ Safe - Pure computation, no I/O

### String
✅ Safe - Pure transformation, no I/O

### HTTP
⚠️ Caution - Can make external requests
- Agents can call any URL
- Consider rate limiting (future)
- Consider domain whitelist/blacklist (future)

### Filesystem
⚠️ Caution - Can access local files
- Should use sandboxing (base_dir)
- Path validation prevents traversal attacks
- Consider read-only mode (future)

## Future Enhancements

### Short Term (Week 5-8)
- HTTP tool: Better error messages
- Filesystem: Read-only mode flag
- All tools: Better input validation

### Medium Term (Week 9-12)
- Rate limiting for HTTP
- Domain whitelist/blacklist for HTTP
- File size limits for Filesystem
- Memory limits for Calculator

### Long Term (Week 13+)
- Tool usage analytics
- Tool performance monitoring
- Tool versioning
- Tool deprecation strategy

## Conclusion

The 4 core tools provide a solid foundation:
- **Calculator** - Math and logic
- **String** - Text processing
- **HTTP** - External integration
- **Filesystem** - Local data

Together, they enable agents to:
✅ Perform calculations
✅ Process text
✅ Access any API
✅ Manage local files

This minimal but complete set keeps the platform simple while enabling powerful agents through composition and the HTTP tool's universal adapter pattern.

---

**Current Status:** ✅ All 4 core tools implemented and tested
**Total Tests:** 36 passing
**Total LOC:** ~1,100 lines
