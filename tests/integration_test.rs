//
//  SPDX-License-Identifier: MIT
//  Copyright (c) 2026 Paal Øye-Strømme
//
//  integration_test.rs
//  jsonrpc-stdio-proxy
//

use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[tokio::test]
async fn test_mcp_style_ndjson() {
    // Spawn proxy with 'cat' to echo back everything
    let mut child = Command::new("cargo")
        .args(["run", "--", "--", "cat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn proxy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let input = b"{\"jsonrpc\": \"2.0\", \"method\": \"test-mcp\"}\n";
    stdin.write_all(input).await.unwrap();
    stdin.flush().await.unwrap();

    let mut buf = vec![0u8; input.len()];
    stdout.read_exact(&mut buf).await.unwrap();

    assert_eq!(buf, input);

    child.kill().await.unwrap();
}

#[tokio::test]
async fn test_lsp_style_header_delimited() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "--", "cat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn proxy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let json_payload = b"{\"jsonrpc\": \"2.0\", \"method\": \"test-lsp\"}";
    let headers = format!("Content-Length: {}\r\n\r\n", json_payload.len());
    let mut input = headers.as_bytes().to_vec();
    input.extend_from_slice(json_payload);

    stdin.write_all(&input).await.unwrap();
    stdin.flush().await.unwrap();

    let mut buf = vec![0u8; input.len()];
    stdout.read_exact(&mut buf).await.unwrap();

    assert_eq!(buf, input);

    child.kill().await.unwrap();
}

#[tokio::test]
async fn test_multiple_messages_mixed() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "--", "cat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn proxy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    // 1. Send MCP message
    let mcp_msg = b"{\"id\": 1, \"method\": \"mcp-1\"}\n";
    stdin.write_all(mcp_msg).await.unwrap();

    // 2. Send LSP message
    let lsp_json = b"{\"id\": 2, \"method\": \"lsp-2\"}";
    let lsp_headers = format!("Content-Length: {}\r\n\r\n", lsp_json.len());
    let mut lsp_msg = lsp_headers.as_bytes().to_vec();
    lsp_msg.extend_from_slice(lsp_json);
    stdin.write_all(&lsp_msg).await.unwrap();
    stdin.flush().await.unwrap();

    // Read and verify MCP back
    let mut buf_mcp = vec![0u8; mcp_msg.len()];
    stdout.read_exact(&mut buf_mcp).await.unwrap();
    assert_eq!(buf_mcp, mcp_msg);

    // Read and verify LSP back
    let mut buf_lsp = vec![0u8; lsp_msg.len()];
    stdout.read_exact(&mut buf_lsp).await.unwrap();
    assert_eq!(buf_lsp, lsp_msg);

    child.kill().await.unwrap();
}
