//! KCode server lifecycle — probe, spawn, capture URL/token.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

const DEFAULT_PORT: u16 = 58627;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Managed Kimi/acode web server handle.
pub struct KcodeServer {
    url: String,
    child: Arc<Mutex<Child>>,
}

impl KcodeServer {
    /// Return the captured authenticated URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check whether the tracked child process is still running.
    pub async fn is_running(&self) -> bool {
        let mut child = self.child.lock().await;
        matches!(child.try_wait(), Ok(None))
    }

    /// Start a new server if none is running, capture its Local URL.
    pub async fn start(port: u16, timeout_secs: u64) -> Result<Self, String> {
        let cli = resolve_cli().ok_or("No coding-agent CLI found (tried kimi, acode).")?;

        let mut child = Command::new(&cli)
            .args(["web", "--port", &port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to start {} web: {}", cli.display(), e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or("no stdout from kcode server")?;
        let mut reader = BufReader::new(stdout).lines();

        let local_url: String = timeout(
            Duration::from_secs(timeout_secs),
            async {
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(url) = parse_local_url(&line) {
                        return Some(url);
                    }
                }
                None
            },
        )
        .await
        .map_err(|_| "timed out waiting for kcode web to print its Local URL".to_string())?
        .ok_or_else(|| "kcode web started but never printed a Local URL".to_string())?;

        Ok(KcodeServer {
            url: local_url,
            child: Arc::new(Mutex::new(child)),
        })
    }

    /// Stop the tracked server.
    pub async fn stop(&self) {
        let mut child = self.child.lock().await;
        let _ = child.start_kill();
    }
}

/// Ensure a server is running and return its authenticated URL.
pub async fn ensure_kcode_url(port: u16, timeout_secs: u64) -> Result<String, String> {
    // 1. Reuse an existing server on the port.
    if is_server_running(port).await {
        let token = read_server_token();
        let base = format!("http://127.0.0.1:{}/", port);
        return Ok(match token {
            Some(t) => format!("{}#token={}", base, t),
            None => base,
        });
    }

    // 2. Start a fresh server.
    let server = KcodeServer::start(port, timeout_secs).await?;
    let url = server.url().to_string();
    // We intentionally drop the server handle; the process keeps running.
    // In production we might keep it in AppState and shut it down on exit.
    Ok(url)
}

/// Probe whether the coding-agent web server is already running.
async fn is_server_running(port: u16) -> bool {
    matches!(
        timeout(
            Duration::from_millis(800),
            tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        )
        .await,
        Ok(Ok(_))
    )
}

/// Discover the local coding-agent CLI: prefer upstream kimi, then AXIOM acode.
/// GUI apps do not inherit the user's shell $PATH, so check known install
/// locations before falling back to PATH.
fn resolve_cli() -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let known = [
        home.join(".kimi-code/bin/kimi"),
        home.join(".local/bin/kimi"),
        PathBuf::from("/usr/local/bin/kimi"),
        PathBuf::from("/opt/homebrew/bin/kimi"),
        home.join(".axiom/bin/acode"),
        home.join(".local/bin/acode"),
        PathBuf::from("/usr/local/bin/acode"),
        PathBuf::from("/opt/homebrew/bin/acode"),
    ];
    for p in known {
        if p.is_file() {
            return Some(p);
        }
    }
    for bin in ["kimi", "acode"] {
        if let Some(p) = std::env::split_paths(&std::env::var("PATH").unwrap_or_default())
            .map(|d| d.join(bin))
            .find(|p| p.is_file())
        {
            return Some(p);
        }
    }
    None
}

/// Parse the Local URL line printed by `acode web` / `kimi web`:
///   Local:    http://127.0.0.1:58627/#token=...
fn parse_local_url(line: &str) -> Option<String> {
    let line = line.trim();
    let idx = line.find("Local:")?;
    let url_part = line[idx + 6..].trim();
    let url = url_part.split_whitespace().next()?;
    if url.starts_with("http://") || url.starts_with("https://") {
        Some(url.to_string())
    } else {
        None
    }
}

/// Resolve the Kimi Code config directory.
fn resolve_kimi_home() -> PathBuf {
    if let Some(home) = std::env::var_os("KIMI_CODE_HOME") {
        return PathBuf::from(home);
    }
    let home = dirs::home_dir().unwrap_or_default();
    if home.join(".kimi-code").is_dir() {
        home.join(".kimi-code")
    } else {
        home.join(".config").join("kimi-code")
    }
}

/// Read the bearer token written by a running `kimi web` / `acode web`.
fn read_server_token() -> Option<String> {
    let token_file = resolve_kimi_home().join("server.token");
    std::fs::read_to_string(&token_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
