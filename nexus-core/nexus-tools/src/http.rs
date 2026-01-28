//! HTTP tool for making web requests

use async_trait::async_trait;
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::{Result, ToolError};
use crate::tool::{Tool, ToolOutput, ToolTimer};

/// HTTP tool for making REST API calls
pub struct HttpTool {
    client: Client,
    #[allow(dead_code)]
    default_timeout: Duration,
}

impl HttpTool {
    /// Create a new HTTP tool with default settings
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(30))
    }

    /// Create a new HTTP tool with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            default_timeout: timeout,
        }
    }

    /// Parse method string to reqwest Method
    fn parse_method(method: &str) -> Result<Method> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            _ => Err(ToolError::InvalidInput(format!(
                "Unsupported HTTP method: {}",
                method
            ))),
        }
    }
}

impl Default for HttpTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http"
    }

    fn description(&self) -> &str {
        "Make HTTP requests (GET, POST, PUT, DELETE, PATCH). \
         Supports custom headers and request body."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                    "description": "HTTP method to use"
                },
                "url": {
                    "type": "string",
                    "description": "URL to request"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional HTTP headers",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "body": {
                    "type": "string",
                    "description": "Optional request body (for POST, PUT, PATCH)"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional timeout in seconds (default: 30)",
                    "minimum": 1,
                    "maximum": 300
                }
            },
            "required": ["method", "url"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let timer = ToolTimer::start();

        // Extract parameters
        let method_str = input["method"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'method' field".to_string()))?;

        let url = input["url"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'url' field".to_string()))?;

        let method = Self::parse_method(method_str)?;

        // Build request
        let mut request = self.client.request(method.clone(), url);

        // Add headers if provided
        if let Some(headers_obj) = input["headers"].as_object() {
            for (key, value) in headers_obj {
                if let Some(value_str) = value.as_str() {
                    request = request.header(key, value_str);
                }
            }
        }

        // Add body if provided (for POST, PUT, PATCH)
        if let Some(body) = input["body"].as_str() {
            request = request.body(body.to_string());
            // Set content-type if not already set
            if input["headers"].is_null()
                || !input["headers"]
                    .as_object()
                    .map(|h| h.contains_key("Content-Type"))
                    .unwrap_or(false)
            {
                request = request.header("Content-Type", "application/json");
            }
        }

        // Execute request
        let response = request.send().await?;

        // Extract metadata
        let status = response.status();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Get response body
        let body = response.text().await?;

        let metadata = json!({
            "status": status.as_u16(),
            "status_text": status.canonical_reason().unwrap_or("Unknown"),
            "headers": headers,
            "url": url,
            "method": method_str,
        });

        let output = ToolOutput::success_with_metadata(body, metadata, timer.elapsed_ms());

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_method() {
        assert!(HttpTool::parse_method("GET").is_ok());
        assert!(HttpTool::parse_method("post").is_ok());
        assert!(HttpTool::parse_method("DELETE").is_ok());
        assert!(HttpTool::parse_method("INVALID").is_err());
    }

    #[test]
    fn test_http_tool_name() {
        let tool = HttpTool::new();
        assert_eq!(tool.name(), "http");
    }

    #[test]
    fn test_http_tool_description() {
        let tool = HttpTool::new();
        assert!(tool.description().contains("HTTP"));
    }

    #[test]
    fn test_http_tool_parameters() {
        let tool = HttpTool::new();
        let params = tool.parameters();
        assert!(params["properties"]["method"].is_object());
        assert!(params["properties"]["url"].is_object());
        assert!(params["required"].is_array());
    }

    #[tokio::test]
    async fn test_http_get_request() {
        let tool = HttpTool::new();
        let input = json!({
            "method": "GET",
            "url": "https://httpbin.org/get"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.content.contains("httpbin"));
        assert!(output.metadata.is_some());

        if let Some(metadata) = output.metadata {
            assert_eq!(metadata["status"], 200);
            assert_eq!(metadata["method"], "GET");
        }
    }

    #[tokio::test]
    async fn test_http_post_request() {
        let tool = HttpTool::new();
        let input = json!({
            "method": "POST",
            "url": "https://httpbin.org/post",
            "body": r#"{"test": "data"}"#
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.content.contains("test"));
    }

    #[tokio::test]
    async fn test_http_invalid_method() {
        let tool = HttpTool::new();
        let input = json!({
            "method": "INVALID",
            "url": "https://httpbin.org/get"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_http_missing_url() {
        let tool = HttpTool::new();
        let input = json!({
            "method": "GET"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_http_with_headers() {
        let tool = HttpTool::new();
        let input = json!({
            "method": "GET",
            "url": "https://httpbin.org/headers",
            "headers": {
                "X-Custom-Header": "test-value"
            }
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.content.contains("X-Custom-Header"));
    }
}
