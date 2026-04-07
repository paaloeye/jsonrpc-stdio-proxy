use clap::Parser;
use log::{error, info};
use oslog::OsLogger;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::signal;

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

    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    // Task: Proxy stdin (parent) -> stdin (child)
    let _stdin_task = tokio::spawn(async move {
        let mut stdin = io::stdin();
        if let Err(e) = io::copy(&mut stdin, &mut child_stdin).await {
            error!("Error copying from stdin to child: {}", e);
        }
    });

    // Task: Proxy stdout (child) -> stdout (parent)
    let stdout_task = tokio::spawn(async move {
        let mut stdout = io::stdout();
        if let Err(e) = io::copy(&mut child_stdout, &mut stdout).await {
            error!("Error copying from child stdout to parent: {}", e);
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
