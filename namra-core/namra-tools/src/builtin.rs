//! Built-in utility tools (calculator, string operations, etc.)

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{Result, ToolError};
use crate::tool::{Tool, ToolOutput, ToolTimer};

/// Calculator tool for arithmetic operations
pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a mathematical expression
    fn evaluate(&self, expression: &str) -> Result<f64> {
        // Simple arithmetic parser
        // Supports: +, -, *, /, (, )
        // Note: This is a basic implementation. For production, use a proper math parser library.

        let expression = expression.replace(" ", "");

        // Try to parse as simple operations first
        if let Some(result) = self.try_simple_operation(&expression) {
            return Ok(result);
        }

        Err(ToolError::InvalidInput(format!(
            "Unable to evaluate expression: {}. Supported: simple arithmetic with +, -, *, /",
            expression
        )))
    }

    /// Try to evaluate simple binary operations
    fn try_simple_operation(&self, expr: &str) -> Option<f64> {
        // Try each operator (in reverse precedence order)
        for op in ['+', '-', '*', '/'] {
            if let Some(pos) = expr.rfind(op) {
                // Skip if it's a negative sign at the start
                if op == '-' && pos == 0 {
                    continue;
                }

                let left = &expr[..pos];
                let right = &expr[pos + 1..];

                if let (Ok(left_val), Ok(right_val)) = (left.parse::<f64>(), right.parse::<f64>()) {
                    return Some(match op {
                        '+' => left_val + right_val,
                        '-' => left_val - right_val,
                        '*' => left_val * right_val,
                        '/' => {
                            if right_val == 0.0 {
                                return None; // Division by zero
                            }
                            left_val / right_val
                        }
                        _ => return None,
                    });
                }
            }
        }

        // Try parsing as a single number
        expr.parse::<f64>().ok()
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform arithmetic calculations. \
         Supports: addition (+), subtraction (-), multiplication (*), division (/)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate (e.g., '2 + 2', '10 * 5', '100 / 4')"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let timer = ToolTimer::start();

        let expression = input["expression"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'expression' field".to_string()))?;

        let result = self.evaluate(expression)?;

        let metadata = json!({
            "expression": expression,
            "result": result,
            "operation": "calculate"
        });

        Ok(ToolOutput::success_with_metadata(
            format!("{} = {}", expression, result),
            metadata,
            timer.elapsed_ms(),
        ))
    }
}

/// String manipulation tool
pub struct StringTool;

impl StringTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StringTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for StringTool {
    fn name(&self) -> &str {
        "string"
    }

    fn description(&self) -> &str {
        "String manipulation operations. \
         Supports: uppercase, lowercase, reverse, length, trim, replace."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["uppercase", "lowercase", "reverse", "length", "trim", "replace"],
                    "description": "String operation to perform"
                },
                "text": {
                    "type": "string",
                    "description": "Input text"
                },
                "find": {
                    "type": "string",
                    "description": "Text to find (for 'replace' operation)"
                },
                "replace_with": {
                    "type": "string",
                    "description": "Replacement text (for 'replace' operation)"
                }
            },
            "required": ["operation", "text"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let timer = ToolTimer::start();

        let operation = input["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'operation' field".to_string()))?;

        let text = input["text"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'text' field".to_string()))?;

        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            "length" => text.len().to_string(),
            "trim" => text.trim().to_string(),
            "replace" => {
                let find = input["find"].as_str().ok_or_else(|| {
                    ToolError::InvalidInput(
                        "Missing 'find' field for replace operation".to_string(),
                    )
                })?;
                let replace_with = input["replace_with"].as_str().ok_or_else(|| {
                    ToolError::InvalidInput(
                        "Missing 'replace_with' field for replace operation".to_string(),
                    )
                })?;
                text.replace(find, replace_with)
            }
            _ => {
                return Err(ToolError::InvalidInput(format!(
                    "Unknown operation: {}",
                    operation
                )))
            }
        };

        let metadata = json!({
            "operation": operation,
            "input_length": text.len(),
            "output_length": result.len(),
        });

        Ok(ToolOutput::success_with_metadata(
            result,
            metadata,
            timer.elapsed_ms(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Calculator tests
    #[test]
    fn test_calculator_tool_name() {
        let tool = CalculatorTool::new();
        assert_eq!(tool.name(), "calculator");
    }

    #[test]
    fn test_calculator_simple_operations() {
        let tool = CalculatorTool::new();
        assert_eq!(tool.evaluate("2+2").unwrap(), 4.0);
        assert_eq!(tool.evaluate("10-5").unwrap(), 5.0);
        assert_eq!(tool.evaluate("3*4").unwrap(), 12.0);
        assert_eq!(tool.evaluate("20/4").unwrap(), 5.0);
    }

    #[test]
    fn test_calculator_with_spaces() {
        let tool = CalculatorTool::new();
        assert_eq!(tool.evaluate("2 + 2").unwrap(), 4.0);
        assert_eq!(tool.evaluate("10 - 5").unwrap(), 5.0);
    }

    #[test]
    fn test_calculator_decimals() {
        let tool = CalculatorTool::new();
        assert_eq!(tool.evaluate("2.5+2.5").unwrap(), 5.0);
        assert_eq!(tool.evaluate("10.5*2").unwrap(), 21.0);
    }

    #[test]
    fn test_calculator_negative_numbers() {
        let tool = CalculatorTool::new();
        assert_eq!(tool.evaluate("-5").unwrap(), -5.0);
    }

    #[tokio::test]
    async fn test_calculator_execute() {
        let tool = CalculatorTool::new();
        let input = json!({
            "expression": "25 * 4"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.content.contains("100"));
    }

    #[tokio::test]
    async fn test_calculator_invalid_expression() {
        let tool = CalculatorTool::new();
        let input = json!({
            "expression": "invalid"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    // String tool tests
    #[test]
    fn test_string_tool_name() {
        let tool = StringTool::new();
        assert_eq!(tool.name(), "string");
    }

    #[tokio::test]
    async fn test_string_uppercase() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "uppercase",
            "text": "hello world"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.content, "HELLO WORLD");
    }

    #[tokio::test]
    async fn test_string_lowercase() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "lowercase",
            "text": "HELLO WORLD"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.content, "hello world");
    }

    #[tokio::test]
    async fn test_string_reverse() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "reverse",
            "text": "hello"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.content, "olleh");
    }

    #[tokio::test]
    async fn test_string_length() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "length",
            "text": "hello"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.content, "5");
    }

    #[tokio::test]
    async fn test_string_trim() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "trim",
            "text": "  hello world  "
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.content, "hello world");
    }

    #[tokio::test]
    async fn test_string_replace() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "replace",
            "text": "hello world",
            "find": "world",
            "replace_with": "Rust"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.content, "hello Rust");
    }

    #[tokio::test]
    async fn test_string_invalid_operation() {
        let tool = StringTool::new();
        let input = json!({
            "operation": "invalid",
            "text": "hello"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }
}
