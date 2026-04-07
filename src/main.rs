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

        let mut payload_to_log = None;
        let mut total_bytes = 0;

        if header_buf.starts_with("Content-Length:") {
            // --- Header-Delimited (LSP/DAP) ---
            let mut content_length: usize = 0;
            total_bytes = bytes_read;

            if let Some(len_str) = header_buf.trim().strip_prefix("Content-Length:") {
                if let Ok(len) = len_str.trim().parse::<usize>() {
                    content_length = len;
                }
            }

            writer.write_all(header_buf.as_bytes()).await?;

            loop {
                header_buf.clear();
                let n = buf_reader.read_line(&mut header_buf).await?;
                if n == 0 { break; }
                total_bytes += n;
                writer.write_all(header_buf.as_bytes()).await?;
                if header_buf == "\r\n" || header_buf == "\n" { break; }
            }

            let mut payload = vec![0u8; content_length];
            buf_reader.read_exact(&mut payload).await?;
            total_bytes += content_length;
            
            payload_to_log = Some(payload);
            writer.write_all(payload_to_log.as_ref().unwrap()).await?;
            writer.flush().await?;

        } else {
            // --- Newline-Delimited (MCP) ---
            total_bytes = bytes_read;
            let trimmed = header_buf.trim();
            if !trimmed.is_empty() {
                payload_to_log = Some(header_buf.as_bytes().to_vec());
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
                            // It's a request (unless it has 'result'/'error', which is rare for client-to-server)
                            if json.get("method").is_some() {
                                metrics.pending_requests.insert(id, Instant::now());
                            }
                        } else {
                            // It's a response (server-to-client)
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
                metrics.client_to_server_bytes.fetch_add(total_bytes as u64, Ordering::Relaxed);
            } else {
                metrics.server_to_client_messages.fetch_add(1, Ordering::Relaxed);
                metrics.server_to_client_bytes.fetch_add(total_bytes as u64, Ordering::Relaxed);
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
    let _stdin_task = tokio::spawn(async move {
        let stdin = io::stdin();
        if let Err(e) = proxy_and_log_stream(stdin, child_stdin, "Client -> Server", m1, true).await {
            error!("Error proxying stdin to child: {}", e);
        }
    });

    let m2 = Arc::clone(&metrics);
    let stdout_task = tokio::spawn(async move {
        let stdout = io::stdout();
        if let Err(e) = proxy_and_log_stream(child_stdout, stdout, "Server -> Client", m2, false).await {
            error!("Error proxying child stdout to parent: {}", e);
        }
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
    }

    let _ = tokio::time::timeout(Duration::from_millis(500), async {
        let _ = tokio::join!(stdout_task, stderr_task);
    }).await;

    // Log Performance Metrics Summary
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

    Ok(())
}
