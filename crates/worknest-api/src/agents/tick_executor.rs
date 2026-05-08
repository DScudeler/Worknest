//! Run a `claude --permission-mode auto -p '/agent-tick'` subprocess for one
//! deployment, capturing stdout/stderr to a per-tick log file.
//!
//! Hidden behind a [`TickExecutor`] trait so unit tests can drop in a fake
//! that doesn't shell out.

use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct TickReport {
    pub success: bool,
    pub summary: String,
    pub stderr_tail: String,
}

/// What the scheduler tells the executor about the upcoming tick. Wrapped
/// in a struct so adding fields later (e.g. context-window budget, custom
/// system prompt) doesn't break trait users.
#[derive(Debug, Clone)]
pub struct TickRequest<'a> {
    pub workspace: &'a Path,
    pub log_path: &'a Path,
    pub timeout: Duration,
    /// Wire model id to pass via `--model`. `None` keeps the user's CLI
    /// default (typically Sonnet).
    pub model: Option<&'a str>,
}

#[async_trait]
pub trait TickExecutor: Send + Sync + 'static {
    async fn run(&self, req: TickRequest<'_>) -> std::io::Result<TickReport>;
}

/// Default impl: spawn `claude` in the workspace dir, headless, with the
/// `agent-tick` slash command.
#[derive(Debug, Clone)]
pub struct ClaudeCliExecutor {
    pub claude_bin: String,
}

#[async_trait]
impl TickExecutor for ClaudeCliExecutor {
    async fn run(&self, req: TickRequest<'_>) -> std::io::Result<TickReport> {
        let TickRequest {
            workspace,
            log_path,
            timeout,
            model,
        } = req;
        let mut cmd = tokio::process::Command::new(&self.claude_bin);
        cmd.arg("--permission-mode").arg("auto");
        if let Some(m) = model {
            cmd.arg("--model").arg(m);
        }
        cmd.arg("-p")
            .arg("/agent-tick")
            .current_dir(workspace)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = cmd.spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdout_buf = tokio::spawn(async move {
            let mut buf = Vec::new();
            if let Some(mut s) = stdout {
                let _ = tokio::io::copy(&mut s, &mut buf).await;
            }
            buf
        });
        let stderr_buf = tokio::spawn(async move {
            let mut buf = Vec::new();
            if let Some(mut s) = stderr {
                let _ = tokio::io::copy(&mut s, &mut buf).await;
            }
            buf
        });

        let exit_status = match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                let _ = child.start_kill();
                return Ok(TickReport {
                    success: false,
                    summary: "tick timed out".into(),
                    stderr_tail: format!("timed out after {timeout:?}"),
                });
            },
        };
        let stdout_bytes = stdout_buf.await.unwrap_or_default();
        let stderr_bytes = stderr_buf.await.unwrap_or_default();

        // Write the combined log file.
        if let Some(parent) = log_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(mut f) = tokio::fs::File::create(log_path).await {
            let _ = f.write_all(b"--- stdout ---\n").await;
            let _ = f.write_all(&stdout_bytes).await;
            let _ = f.write_all(b"\n--- stderr ---\n").await;
            let _ = f.write_all(&stderr_bytes).await;
        }

        let stdout_text = String::from_utf8_lossy(&stdout_bytes);
        let summary = stdout_text
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();
        let stderr_text = String::from_utf8_lossy(&stderr_bytes);
        let stderr_tail: String = stderr_text
            .lines()
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        Ok(TickReport {
            success: exit_status.success(),
            summary,
            stderr_tail,
        })
    }
}
