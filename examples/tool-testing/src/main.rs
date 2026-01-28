//! Manual tool testing example
//!
//! This demonstrates how to use the tools directly.
//!
//! Run with:
//! cargo run --example test_tools_manual

use nexus_tools::{CalculatorTool, FileSystemTool, HttpTool, StringTool, Tool};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nexus Tools Manual Testing ===\n");

    // Test 1: Calculator Tool
    println!("1. Testing Calculator Tool");
    println!("   Expression: 25 * 4");

    let calc_tool = CalculatorTool::new();
    let calc_result = calc_tool
        .execute(json!({
            "expression": "25 * 4"
        }))
        .await?;

    println!("   Result: {}", calc_result.content);
    println!("   Success: {}", calc_result.success);
    println!("   Time: {}ms\n", calc_result.execution_time_ms);

    // Test 2: String Tool
    println!("2. Testing String Tool");
    println!("   Operation: uppercase");
    println!("   Text: hello from nexus");

    let string_tool = StringTool::new();
    let string_result = string_tool
        .execute(json!({
            "operation": "uppercase",
            "text": "hello from nexus"
        }))
        .await?;

    println!("   Result: {}", string_result.content);
    println!("   Time: {}ms\n", string_result.execution_time_ms);

    // Test 3: HTTP Tool (real API call)
    println!("3. Testing HTTP Tool");
    println!("   Method: GET");
    println!("   URL: https://httpbin.org/get");

    let http_tool = HttpTool::new();
    let http_result = http_tool
        .execute(json!({
            "method": "GET",
            "url": "https://httpbin.org/get"
        }))
        .await?;

    println!("   Success: {}", http_result.success);
    println!("   Response length: {} bytes", http_result.content.len());
    println!("   Time: {}ms", http_result.execution_time_ms);

    if let Some(metadata) = &http_result.metadata {
        println!("   Status: {}", metadata["status"]);
    }
    println!();

    // Test 4: Filesystem Tool
    println!("4. Testing Filesystem Tool");

    // Create temp file
    let temp_path = "/tmp/nexus_test.txt";
    let fs_tool = FileSystemTool::new();

    println!("   Writing file: {}", temp_path);
    let write_result = fs_tool
        .execute(json!({
            "operation": "write",
            "path": temp_path,
            "content": "Hello from Nexus tools!"
        }))
        .await?;

    println!("   {}", write_result.content);
    println!("   Time: {}ms\n", write_result.execution_time_ms);

    println!("   Reading file: {}", temp_path);
    let read_result = fs_tool
        .execute(json!({
            "operation": "read",
            "path": temp_path
        }))
        .await?;

    println!("   Content: {}", read_result.content);
    println!("   Time: {}ms\n", read_result.execution_time_ms);

    // Clean up
    println!("   Deleting file: {}", temp_path);
    let delete_result = fs_tool
        .execute(json!({
            "operation": "delete",
            "path": temp_path
        }))
        .await?;

    println!("   {}", delete_result.content);
    println!();

    println!("=== All Tools Tested Successfully! ===");

    Ok(())
}
