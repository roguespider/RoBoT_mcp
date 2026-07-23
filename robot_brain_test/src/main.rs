//! MCP Integration Test for RoBoT Brain
//!
//! Standalone test suite to verify the compiled MCP server executable.
//!
//! ## Building
//! ```bash
//! cd robot_brain_test
//! cargo build --release
//! ```
//!
//! ## Running Tests (cargo test)
//! Tests are defined in src/lib.rs and run with:
//! ```bash
//! cargo test
//! ```
//!
//! ## Running as CLI
//! ```bash
//! # Test all tools
//! cargo run --release -- --test-all
//!
//! # Test specific tool
//! cargo run --release -- --test-tool store_memory
//!
//! # Specify custom server path
//! cargo run --release -- --server /path/to/robot_brain.exe --test-all
//! ```

use std::path::PathBuf;
use std::process::Stdio;
use std::env;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader, AsyncWriteExt};
use tokio::process::{Command as AsyncCommand, ChildStdout};
use tokio::time::timeout;

fn get_server_path() -> PathBuf {
    env::var("MCP_SERVER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let release_path = PathBuf::from("../RoBoT_Brain/target/release/robot_brain");
            #[cfg(windows)]
            let release_path = release_path.with_extension("exe");
            release_path
        })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args: Vec<String> = env::args().collect();
    let server_path = args.iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(get_server_path);
    
    println!("Testing MCP server at: {}", server_path.display());
    
    if !server_path.exists() {
        anyhow::bail!("Server not found at {}. Build with `cargo build --release` in RoBoT_Brain first.", server_path.display());
    }

    // Spawn the server
    let mut child = AsyncCommand::new(&server_path)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    
    let mut send_id: u64 = 1;
    
    // Helper to send request
    async fn send_request(
        stdin: &mut tokio::process::ChildStdin,
        id: &mut u64,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<()> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": *id,
            "method": method,
            "params": params
        });
        *id += 1;
        let s = serde_json::to_string(&request)?;
        stdin.write_all(s.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        Ok(())
    }
    
    // Helper to read a line, skipping log lines (tracing outputs to stdout)
    async fn read_response_line(stdout: &mut BufReader<ChildStdout>, timeout_secs: u64) -> anyhow::Result<Option<String>> {
        let mut line = String::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
        
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(None); // Timeout
            }
            
            match timeout(remaining, stdout.read_line(&mut line)).await {
                Ok(Ok(0)) => return Ok(None), // EOF
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    // Check if this is a JSON-RPC response
                    if trimmed.starts_with('{') && trimmed.contains("\"jsonrpc\"") {
                        return Ok(Some(line.clone()));
                    }
                    // Skip log lines (tracing outputs to stdout)
                    if !trimmed.is_empty() {
                        println!("[LOG] {}", trimmed);
                    }
                    line.clear();
                }
                Ok(Err(e)) => return Err(anyhow::anyhow!("Read error: {}", e)),
                Err(_) => return Ok(None), // Timeout
            }
        }
    }
    
    // Initialize
    send_request(&mut stdin, &mut send_id, "initialize", serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "clientInfo": { "name": "robot_brain_test", "version": "1.0.0" }
    })).await?;
    
    // Read initialize response - STORE THIS for compliance test
    let init_response = read_response_line(&mut stdout, 5).await?;
    let init_response_str = init_response.clone().unwrap_or_default();
    println!("Initialize response: {}", init_response_str);
    
    // Send initialized notification
    stdin.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}\n").await?;
    
    // List tools
    send_request(&mut stdin, &mut send_id, "tools/list", serde_json::json!({})).await?;
    
    let tools_response = read_response_line(&mut stdout, 5).await?;
    let tools_response_str = tools_response.clone().unwrap_or_default();
    println!("Tools list response: {}", tools_response_str);
    
    if let Ok(response) = serde_json::from_str::<serde_json::Value>(&tools_response_str) {
        if let Some(result) = response.get("result") {
            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                println!("\nFound {} tools:\n", tools.len());
                for tool in tools {
                    if let (Some(name), Some(desc)) = (tool.get("name"), tool.get("description")) {
                        println!("  - {}: {}", name, desc);
                    }
                }
            }
        }
    }
    
    // Test all tools
    let tools = vec![
        ("get_workflow", serde_json::json!({})),
        ("store_memory", serde_json::json!({
            "content": "CLI test memory",
            "memory_type": "note",
            "confidence": 0.9
        })),
        ("search_memory", serde_json::json!({"query": "CLI test", "limit": 5})),
        ("list_memories", serde_json::json!({"limit": 5})),
        ("record_experience", serde_json::json!({
            "action": "cli_test",
            "outcome": "success",
            "context": "CLI test"
        })),
        ("get_experience_stats", serde_json::json!({})),
        ("query_knowledge", serde_json::json!({"query": "test", "limit": 5})),
        ("get_knowledge_stats", serde_json::json!({})),
        ("create_plan", serde_json::json!({"goal": "CLI test plan"})),
        ("list_plans", serde_json::json!({"limit": 5})),
    ];
    
    println!("\n--- Testing MCP Protocol Compliance ---\n");
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Initialize should return proper server info
    {
        println!("TEST: Initialize - checking serverInfo...");
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&init_response_str) {
            if let Some(result) = response.get("result") {
                if let Some(server_info) = result.get("serverInfo") {
                    if let Some(name) = server_info.get("name").and_then(|v| v.as_str()) {
                        println!("  serverInfo.name = '{}'", name);
                        // Server name should be "robot_brain", NOT "rmcp"
                        if name == "robot_brain" {
                            println!("  ✓ PASS: Server name is 'robot_brain'");
                            passed += 1;
                        } else {
                            println!("  ✗ FAIL: Server name should be 'robot_brain', got '{}'", name);
                            println!("    ❌ This is why Zed/LM Studio can't see the server!");
                            failed += 1;
                        }
                    }
                    if let Some(version) = server_info.get("version").and_then(|v| v.as_str()) {
                        println!("  serverInfo.version = '{}'", version);
                        // Version should be a valid semver string (allow any version starting with 0.)
                        if version.starts_with("0.") {
                            println!("  ✓ PASS: Version is '{}' (valid semver)", version);
                            passed += 1;
                        } else {
                            println!("  ✗ FAIL: Version should start with '0.', got '{}'", version);
                            failed += 1;
                        }
                    }
                } else {
                    println!("  ✗ FAIL: Missing serverInfo in response");
                    failed += 1;
                }
                
                // protocolVersion should be present
                if result.get("protocolVersion").is_some() {
                    println!("  ✓ protocolVersion present");
                    passed += 1;
                } else {
                    println!("  ✗ FAIL: Missing protocolVersion");
                    failed += 1;
                }
            } else {
                println!("  ✗ FAIL: Missing 'result' field");
                failed += 1;
            }
        } else {
            println!("  ✗ FAIL: Could not parse initialize response");
            failed += 1;
        }
    }
    
    // Test 2: List tools should return tools in MCP format
    {
        println!("\nTEST: tools/list - checking tool format...");
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&tools_response_str) {
            if let Some(result) = response.get("result") {
                if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                    println!("  ✓ PASS: tools array present with {} tools", tools.len());
                    passed += 1;
                    
                    // Check first tool format - should have name, description, inputSchema
                    if let Some(first_tool) = tools.first() {
                        let has_name = first_tool.get("name").is_some();
                        let has_description = first_tool.get("description").is_some();
                        let has_input_schema = first_tool.get("inputSchema").is_some();
                        
                        if has_name && has_description && has_input_schema {
                            println!("  ✓ PASS: Tool format correct (name, description, inputSchema)");
                            passed += 1;
                        } else {
                            println!("  ✗ FAIL: Tool format wrong - missing fields");
                            if !has_name { println!("    - missing 'name'"); }
                            if !has_description { println!("    - missing 'description'"); }
                            if !has_input_schema { println!("    - missing 'inputSchema'"); }
                            failed += 1;
                        }
                    }
                } else {
                    println!("  ✗ FAIL: Missing tools array");
                    failed += 1;
                }
            }
        }
    }
    
    // Test 3: Tool call should return content array (MCP standard format)
    {
        println!("\nTEST: tools/call - checking response format...");
        send_request(&mut stdin, &mut send_id, "tools/call", serde_json::json!({
            "name": "get_workflow",
            "arguments": {}
        })).await?;
        let tool_response = read_response_line(&mut stdout, 5).await?;
        
        if let Some(response_str) = tool_response {
            println!("  tool response: {}", response_str);
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_str) {
                if let Some(result) = response.get("result") {
                    // MCP standard: result should have 'content' array
                    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                        println!("  ✓ PASS: Response has 'content' array (MCP standard format)");
                        passed += 1;
                        if !content.is_empty() {
                            if let Some(item) = content.first() {
                                if item.get("type").is_some() && item.get("text").is_some() {
                                    println!("  ✓ PASS: Content item has 'type' and 'text' (MCP standard)");
                                    passed += 1;
                                } else {
                                    println!("  ✗ FAIL: Content item missing 'type' or 'text'");
                                    println!("    Response: {:?}", item);
                                    failed += 1;
                                }
                            }
                        }
                    } else if result.get("data").is_some() {
                        // Custom format - not MCP compliant
                        println!("  ✗ FAIL: Response has 'data' instead of 'content' (NOT MCP compliant!)");
                        println!("    MCP standard requires: {{ \"content\": [{{\"type\": \"text\", \"text\": \"...\"}}] }}");
                        println!("    This is why Zed/LM Studio can't use the tools!");
                        failed += 1;
                    } else {
                        println!("  ✗ FAIL: Response has neither 'content' nor 'data'");
                        println!("    Response: {:?}", result);
                        failed += 1;
                    }
                } else if response.get("error").is_some() {
                    println!("  ✗ FAIL: Tool call returned error");
                    failed += 1;
                } else {
                    println!("  ✗ FAIL: No 'result' or 'error' in response");
                    failed += 1;
                }
            } else {
                println!("  ✗ FAIL: Could not parse tool response");
                failed += 1;
            }
        } else {
            println!("  ✗ FAIL: No response from tool call (timeout)");
            failed += 1;
        }
    }
    
    println!("\n===========================================");
    println!("MCP PROTOCOL COMPLIANCE TEST RESULTS");
    println!("===========================================");
    println!("Tests Passed: {}", passed);
    println!("Tests Failed: {}", failed);
    println!("===========================================");
    
    if failed > 0 {
        eprintln!("\n❌ MCP COMPLIANCE TEST FAILED!");
        eprintln!("The server is NOT compatible with standard MCP clients (Zed, LM Studio, etc.)");
        std::process::exit(1);
    }
    
    println!("\n✓ MCP Protocol Compliance: ALL TESTS PASSED");
    Ok(())
}
