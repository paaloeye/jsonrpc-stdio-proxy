use clap::Parser;
use dashmap::DashMap;
use log::{debug, error, info};
use oslog::OsLogger;
use serde_json::Value;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::signal;

struct Metrics {
    start_time: Instant,
    client_to_server_messages: AtomicU64,
    client_to_server_bytes: AtomicU64,
    server_to_client_messages: AtomicU64,
    server_to_client_bytes: AtomicU64,
    errors: AtomicU64,

    // Latency tracking (Request ID -> Start Instant)
    pending_requests: DashMap<String, Instant>,
    latencies: dashmap::DashSet<Duration>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            client_to_server_messages: AtomicU64::new(0),
            client_to_server_bytes: AtomicU64::new(0),
            server_to_client_messages: AtomicU64::new(0),
            server_to_client_bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            pending_requests: DashMap::new(),
            latencies: dashmap::DashSet::new(),
        }
    }
}

async fn proxy_and_log_stream<R, W>(
    reader: R,
    mut writer: W,
    direction_tag: &str,
    metrics: Arc<Metrics>,
    is_client_to_server: bool,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf_reader = BufReader::new(reader);
    let mut header_buf = String::new();

    loop {
        header_buf.clear();
        let bytes_read = match buf_reader.read_line(&mut header_buf).await {
            Ok(n) => n,
            Err(e) => {
                metrics.errors.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
        };

        if bytes_read == 0 {
            break; // EOF
        }

        let payload_to_log: Option<Vec<u8>>;
        let current_message_bytes: usize;

        if header_buf.starts_with("Content-Length:") {
            // --- Header-Delimited (LSP/DAP) ---
            let mut content_length: usize = 0;
            let mut bytes_acc = bytes_read;

            match header_buf.trim().strip_prefix("Content-Length:") {
                Some(len_str) => {
                    if let Ok(len) = len_str.trim().parse::<usize>() {
                        content_length = len;
                    }
                }
                None => todo!(),
            }

            writer.write_all(header_buf.as_bytes()).await?;

            loop {
                header_buf.clear();
                let n = buf_reader.read_line(&mut header_buf).await?;
                if n == 0 { break; }
                bytes_acc += n;
                writer.write_all(header_buf.as_bytes()).await?;
                if header_buf == "\r\n" || header_buf == "\n" { break; }
            }

            let mut payload = vec![0u8; content_length];
            buf_reader.read_exact(&mut payload).await?;
            bytes_acc += content_length;

            writer.write_all(&payload).await?;
            writer.flush().await?;

            current_message_bytes = bytes_acc;
            payload_to_log = Some(payload);

        } else {
            // --- Newline-Delimited (MCP) ---
            current_message_bytes = bytes_read;
            let trimmed = header_buf.trim();
            if !trimmed.is_empty() {
                payload_to_log = Some(header_buf.as_bytes().to_vec());
            } else {
                payload_to_log = None;
            }
            writer.write_all(header_buf.as_bytes()).await?;
            writer.flush().await?;
        }

        // Process Timing and Logging
        if let Some(payload_bytes) = payload_to_log {
            if let Ok(payload_str) = std::str::from_utf8(&payload_bytes) {
                let trimmed_payload = payload_str.trim();
                info!("[{}] {}", direction_tag, trimmed_payload);

                // Latency tracking
                if let Ok(json) = serde_json::from_str::<Value>(trimmed_payload) {
                    if let Some(id_val) = json.get("id") {
                        let id = id_val.to_string();
                        if is_client_to_server {
                            if json.get("method").is_some() && json.get("result").is_none() && json.get("error").is_none() {
                                metrics.pending_requests.insert(id, Instant::now());
                            }
                        } else {
                            if json.get("result").is_some() || json.get("error").is_some() {
                                if let Some((_, start)) = metrics.pending_requests.remove(&id) {
                                    metrics.latencies.insert(start.elapsed());
                                }
                            }
                        }
                    }
                }
            } else {
                debug!("[{}] <binary payload>", direction_tag);
            }

            // Update Global Metrics
            if is_client_to_server {
                metrics.client_to_server_messages.fetch_add(1, Ordering::Relaxed);
                metrics.client_to_server_bytes.fetch_add(current_message_bytes as u64, Ordering::Relaxed);
            } else {
                metrics.server_to_client_messages.fetch_add(1, Ordering::Relaxed);
                metrics.server_to_client_bytes.fetch_add(current_message_bytes as u64, Ordering::Relaxed);
            }
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// OSLog subsystem
    #[arg(short, long, default_value = "com.paaloeye.jsonrpc-proxy")]
    subsystem: String,

    /// OSLog category
    #[arg(short, long, default_value = "default")]
    category: String,

    /// The command and arguments to run
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let metrics = Arc::new(Metrics::default());

    OsLogger::new(&args.subsystem)
        .level_filter(log::LevelFilter::Debug)
        .init()?;

    info!("Starting proxy for command: {:?}", args.command);

    let mut child = Command::new(&args.command[0])
        .args(&args.command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            error!("Failed to spawn child process: {}", e);
            e
        })?;

    let child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    let m1 = Arc::clone(&metrics);
    let mut stdin_task = tokio::spawn(async move {
        let stdin = io::stdin();
        proxy_and_log_stream(stdin, child_stdin, "Client -> Server", m1, true).await
    });

    let m2 = Arc::clone(&metrics);
    let stdout_task = tokio::spawn(async move {
        let stdout = io::stdout();
        proxy_and_log_stream(child_stdout, stdout, "Server -> Client", m2, false).await
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            info!("[child-stderr] {}", line);
        }
    });

    tokio::select! {
        status = child.wait() => {
            match status {
                Ok(s) => info!("Child process exited with status: {}", s),
                Err(e) => error!("Error waiting for child process: {}", e),
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, terminating child process...");
            let _ = child.kill().await;
        }
        _ = &mut stdin_task => {
            info!("Stdin closed, sending SIGTERM to child and waiting...");
            // Standard proxy behavior: if the client closes stdin, the proxy is done.
            // We should kill/terminate the child so it doesn't hang indefinitely.
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }

    // Capture metrics immediately after child exit
    let session_duration = metrics.start_time.elapsed();
    let latencies: Vec<Duration> = metrics.latencies.iter().map(|d| *d).collect();

    info!("--- Performance Metrics Summary ---");
    info!("Session Duration: {:?}", session_duration);
    info!("Client -> Server: {} msgs, {} bytes",
        metrics.client_to_server_messages.load(Ordering::Relaxed),
        metrics.client_to_server_bytes.load(Ordering::Relaxed));
    info!("Server -> Client: {} msgs, {} bytes",
        metrics.server_to_client_messages.load(Ordering::Relaxed),
        metrics.server_to_client_bytes.load(Ordering::Relaxed));

    if !latencies.is_empty() {
        let min = latencies.iter().min().unwrap();
        let max = latencies.iter().max().unwrap();
        let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        info!("RTT Latency: Min {:?}, Max {:?}, Avg {:?}", min, max, avg);
    }
    info!("Errors: {}", metrics.errors.load(Ordering::Relaxed));

    // Wait briefly for remaining stdout/stderr logs
    let _ = tokio::time::timeout(Duration::from_millis(200), async {
        let _ = tokio::join!(stdout_task, stderr_task);
    }).await;

    info!("Proxy exiting");
    Ok(())
}
