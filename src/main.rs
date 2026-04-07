use clap::Parser;
use log::{error, info, debug};
use oslog::OsLogger;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::signal;

async fn proxy_and_log_stream<R, W>(
    reader: R,
    mut writer: W,
    direction_tag: &str,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf_reader = BufReader::new(reader);
    let mut header_buf = String::new();

    loop {
        header_buf.clear();
        // Read the first line to determine framing
        let bytes_read = buf_reader.read_line(&mut header_buf).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        if header_buf.starts_with("Content-Length:") {
            // --- Header-Delimited (LSP/DAP) ---
            let mut content_length: usize = 0;
            
            // Parse the Content-Length value
            if let Some(len_str) = header_buf.trim().strip_prefix("Content-Length:") {
                if let Ok(len) = len_str.trim().parse::<usize>() {
                    content_length = len;
                }
            }

            // Write the first header line to the destination
            writer.write_all(header_buf.as_bytes()).await?;

            // Read the rest of the headers until \r\n\r\n
            loop {
                header_buf.clear();
                let n = buf_reader.read_line(&mut header_buf).await?;
                if n == 0 {
                    break;
                }
                writer.write_all(header_buf.as_bytes()).await?;
                
                if header_buf == "\r\n" || header_buf == "\n" {
                    break; // End of headers
                }
            }

            // Read the exact payload
            let mut payload = vec![0u8; content_length];
            buf_reader.read_exact(&mut payload).await?;
            
            // Log and forward
            if let Ok(payload_str) = std::str::from_utf8(&payload) {
                info!("[{}] {}", direction_tag, payload_str);
            } else {
                debug!("[{}] <binary payload of {} bytes>", direction_tag, content_length);
            }
            writer.write_all(&payload).await?;
            writer.flush().await?;

        } else {
            // --- Newline-Delimited (MCP) or raw text ---
            let trimmed = header_buf.trim();
            if !trimmed.is_empty() {
                info!("[{}] {}", direction_tag, trimmed);
            }
            writer.write_all(header_buf.as_bytes()).await?;
            writer.flush().await?;
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

    // Initialize OSLog
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

    // Task: Proxy stdin (parent) -> stdin (child)
    let _stdin_task = tokio::spawn(async move {
        let stdin = io::stdin();
        if let Err(e) = proxy_and_log_stream(stdin, child_stdin, "Client -> Server").await {
            error!("Error proxying stdin to child: {}", e);
        }
        // child_stdin will be dropped here, closing the pipe to the child process
    });

    // Task: Proxy stdout (child) -> stdout (parent)
    let stdout_task = tokio::spawn(async move {
        let stdout = io::stdout();
        if let Err(e) = proxy_and_log_stream(child_stdout, stdout, "Server -> Client").await {
            error!("Error proxying child stdout to parent: {}", e);
        }
    });

    // Task: Proxy stderr (child) -> OSLog
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stderr).lines();
        while let Ok(line_result) = reader.next_line().await {
            match line_result {
                Some(line) => info!("[child-stderr] {}", line),
                None => break,
            }
        }
    });

    // Handle termination
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

    // Wait for stdout and stderr tasks to finish flushing (with a timeout)
    let _ = tokio::time::timeout(std::time::Duration::from_millis(500), async {
        let _ = tokio::join!(stdout_task, stderr_task);
    }).await;

    Ok(())
}
