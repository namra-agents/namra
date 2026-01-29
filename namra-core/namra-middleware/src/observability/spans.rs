//! Span creation helpers for tracing

use tracing::Span;

/// Default maximum content size for span attributes (OTEL typical limit)
const DEFAULT_MAX_CONTENT_SIZE: usize = 4000;

/// Create a span for an agent run
pub fn agent_run_span(agent_name: &str, agent_version: Option<&str>) -> Span {
    tracing::info_span!(
        "agent.run",
        otel.kind = "internal",
        agent.name = %agent_name,
        agent.version = %agent_version.unwrap_or("unknown"),
        agent.iterations = tracing::field::Empty,
        agent.success = tracing::field::Empty,
    )
}

/// Create a span for an LLM request with content placeholders
pub fn llm_request_span(provider: &str, model: &str) -> Span {
    tracing::info_span!(
        "llm.request",
        otel.kind = "client",
        llm.provider = %provider,
        llm.model = %model,
        llm.tokens.input = tracing::field::Empty,
        llm.tokens.output = tracing::field::Empty,
        llm.cost = tracing::field::Empty,
        // Content fields (recorded when capture_content is enabled)
        llm.prompts = tracing::field::Empty,
        llm.response = tracing::field::Empty,
    )
}

/// Create a span for tool execution with input/output placeholders
pub fn tool_execution_span(tool_name: &str) -> Span {
    tracing::info_span!(
        "tool.execute",
        otel.kind = "internal",
        tool.name = %tool_name,
        tool.success = tracing::field::Empty,
        tool.duration_ms = tracing::field::Empty,
        // Content fields (recorded when capture_content is enabled)
        tool.input = tracing::field::Empty,
        tool.output = tracing::field::Empty,
    )
}

/// Record LLM response metrics on a span
pub fn record_llm_metrics(span: &Span, input_tokens: u32, output_tokens: u32, cost: f64) {
    span.record("llm.tokens.input", input_tokens);
    span.record("llm.tokens.output", output_tokens);
    span.record("llm.cost", cost);
}

/// Record LLM prompt content on a span
/// Content will be truncated if it exceeds max_size
pub fn record_llm_prompts(span: &Span, prompts: &str, max_size: usize) {
    let truncated = truncate_content(prompts, max_size);
    span.record("llm.prompts", truncated.as_str());
}

/// Record LLM response content on a span
/// Content will be truncated if it exceeds max_size
pub fn record_llm_response(span: &Span, response: &str, max_size: usize) {
    let truncated = truncate_content(response, max_size);
    span.record("llm.response", truncated.as_str());
}

/// Record tool execution result on a span
pub fn record_tool_result(span: &Span, success: bool, duration_ms: u64) {
    span.record("tool.success", success);
    span.record("tool.duration_ms", duration_ms);
}

/// Record tool input on a span
/// Content will be truncated if it exceeds max_size
pub fn record_tool_input(span: &Span, input: &str, max_size: usize) {
    let truncated = truncate_content(input, max_size);
    span.record("tool.input", truncated.as_str());
}

/// Record tool output on a span
/// Content will be truncated if it exceeds max_size
pub fn record_tool_output(span: &Span, output: &str, max_size: usize) {
    let truncated = truncate_content(output, max_size);
    span.record("tool.output", truncated.as_str());
}

/// Record agent execution result on a span
pub fn record_agent_result(span: &Span, iterations: u32, success: bool) {
    span.record("agent.iterations", iterations);
    span.record("agent.success", success);
}

/// Truncate content to fit within OTEL attribute size limits
fn truncate_content(content: &str, max_size: usize) -> String {
    let max_size = if max_size == 0 { DEFAULT_MAX_CONTENT_SIZE } else { max_size };

    if content.len() <= max_size {
        content.to_string()
    } else {
        // Truncate at a safe boundary (UTF-8 aware)
        let truncate_at = max_size.saturating_sub(12); // Room for "[TRUNCATED]"
        let mut result = String::with_capacity(max_size);

        for (i, c) in content.char_indices() {
            if i >= truncate_at {
                break;
            }
            result.push(c);
        }

        result.push_str("[TRUNCATED]");
        result
    }
}
