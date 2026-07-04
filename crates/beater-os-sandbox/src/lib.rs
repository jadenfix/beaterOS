//! `beater-os-sandbox` — a scoped local execution lane with observed-effect
//! receipts.
//!
//! This is the first, deliberately-honest slice of `final.md` §10.6 (Sandbox
//! Service) and MVP §24 item 6 ("run tools in a sandbox"). It runs a single
//! shell/program action and, per §7.4 and §26, binds a receipt to the effects
//! it *observes* rather than to anything the agent claims.
//!
//! What this lane guarantees today:
//! - **No inherited ambient authority.** The child process is spawned with
//!   `env_clear()`, so no parent environment secret (tokens, `PATH`, `HOME`,
//!   cloud creds) leaks in. Only an explicit env allowlist is passed.
//! - **Scoped working directory.** Execution `current_dir`s into the granted
//!   workspace; stdin is `/dev/null`.
//! - **Bounded by construction.** Output is capped (`max_output_bytes`),
//!   execution is capped (`timeout`, enforced by kill), and the workspace scan
//!   is capped (`MAX_WORKSPACE_FILES`) — no unbounded memory/CPU from untrusted
//!   input.
//! - **Observed-effect receipt.** The workspace file set + content hashes are
//!   snapshotted before and after; the diff is the observed filesystem side
//!   effect, recorded on a `CapabilityReceiptInput` bound to observed values
//!   (exit status, stdout digest, changed paths) — never self-reported.
//!
//! What this lane does NOT yet do (labeled future hardening, not overclaimed):
//! OS-level *confinement* that prevents a process from escaping the workspace
//! (writing outside it, opening the network, reading other files). That needs
//! macOS `sandbox-exec`/Seatbelt or Linux seccomp+namespaces+cgroups (§10.6
//! Container/VM lanes) and belongs behind a confinement trait seam in a later
//! slice. Until then, effects *outside* the workspace are not observed; callers
//! must treat this as ambient-authority reduction + observation, not isolation.

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use beater_os_core::{CapabilityReceiptInput, CapabilitySelector, SideEffectClass};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Default wall-clock cap for a single action.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
/// Default cap on captured stdout/stderr bytes (each stream, bounded memory).
pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
/// Cap on files scanned per workspace snapshot; above this we fail closed
/// rather than walk an unbounded tree.
pub const MAX_WORKSPACE_FILES: usize = 100_000;

const POLL_INTERVAL: Duration = Duration::from_millis(5);
const HASH_CHUNK: usize = 64 * 1024;

/// A scoped shell/program action to run in the lane.
#[derive(Clone, Debug)]
pub struct ScopedShellAction {
    /// Action id this execution fulfills (bound into the receipt).
    pub action_id: String,
    /// Tool identity for the receipt (e.g. `tool:shell`).
    pub tool_id: String,
    /// The capability target this action was admitted against.
    pub target: CapabilitySelector,
    /// Program to execute (an explicit path is recommended, e.g. `/bin/sh`).
    pub program: String,
    /// Arguments passed to the program.
    pub args: Vec<String>,
    /// Granted workspace root; execution is scoped here and effects are observed
    /// under it.
    pub workspace: PathBuf,
    /// Explicit environment allowlist. The inherited environment is cleared;
    /// only these are passed through.
    pub env: Vec<(String, String)>,
    /// Wall-clock cap; the process is killed past it and the run is `timed_out`.
    pub timeout: Duration,
    /// Per-stream cap on captured output bytes.
    pub max_output_bytes: usize,
}

impl ScopedShellAction {
    /// A new action with default bounds and an empty (cleared) environment.
    pub fn new(
        action_id: impl Into<String>,
        tool_id: impl Into<String>,
        target: CapabilitySelector,
        workspace: impl Into<PathBuf>,
        program: impl Into<String>,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            tool_id: tool_id.into(),
            target,
            program: program.into(),
            args: Vec::new(),
            workspace: workspace.into(),
            env: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }

    /// Append a program argument.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add one explicitly-allowed environment variable (nothing else is passed).
    #[must_use]
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Override the wall-clock timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// The kind of an observed filesystem change under the workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FsChangeKind {
    Created,
    Modified,
    Deleted,
}

/// A single observed filesystem change, relative to the workspace root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FsChange {
    pub path: String,
    pub kind: FsChangeKind,
}

/// The result of running an action: the observed receipt plus raw detail.
#[derive(Clone, Debug)]
pub struct SandboxOutcome {
    /// Receipt input bound to observed values; append it to a `ReceiptLedger`.
    pub receipt_input: CapabilityReceiptInput,
    /// Process exit code, if it exited normally (`None` if killed/timed out).
    pub exit_code: Option<i32>,
    /// Whether the process was killed for exceeding the timeout.
    pub timed_out: bool,
    /// Captured stdout (truncated to `max_output_bytes`).
    pub stdout: Vec<u8>,
    /// Captured stderr (truncated to `max_output_bytes`).
    pub stderr: Vec<u8>,
    /// Observed filesystem changes under the workspace, sorted by path.
    pub fs_changes: Vec<FsChange>,
}

/// Failure modes of the lane. Every one is fail-closed: no `SandboxOutcome`
/// (and therefore no success receipt) is produced on error.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("workspace does not exist: {0}")]
    WorkspaceMissing(PathBuf),
    #[error("workspace is not a directory: {0}")]
    WorkspaceNotDir(PathBuf),
    #[error("workspace exceeds the {limit}-file scan cap; refusing to run")]
    WorkspaceTooLarge { limit: usize },
    #[error("could not spawn sandboxed process: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("sandbox I/O error: {0}")]
    Io(#[source] std::io::Error),
    #[error("sandbox output-drain thread panicked")]
    OutputThreadPanicked,
}

/// Run `action` in the scoped lane and return the observed outcome.
pub fn run(action: &ScopedShellAction) -> Result<SandboxOutcome, SandboxError> {
    let workspace = action.workspace.as_path();
    match std::fs::symlink_metadata(workspace) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return Err(SandboxError::WorkspaceNotDir(workspace.to_path_buf())),
        Err(_) => return Err(SandboxError::WorkspaceMissing(workspace.to_path_buf())),
    }

    let before = snapshot(workspace)?;
    let started_at = Utc::now();

    let mut child = Command::new(&action.program)
        .args(&action.args)
        .env_clear()
        .envs(action.env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SandboxError::Spawn)?;

    // Drain both streams on their own threads so a chatty child cannot deadlock
    // by filling a pipe buffer while we poll for the timeout.
    let cap = action.max_output_bytes;
    let stdout_thread = child.stdout.take().map(|h| drain(h, cap));
    let stderr_thread = child.stderr.take().map(|h| drain(h, cap));

    let deadline = Instant::now() + action.timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(SandboxError::Io)? {
            Some(status) => break Some(status),
            None => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        }
    };

    let stdout = join_drain(stdout_thread)?;
    let stderr = join_drain(stderr_thread)?;
    let finished_at = Utc::now();

    let after = snapshot(workspace)?;
    let fs_changes = diff(&before, &after);

    let exit_code = status.and_then(|status| status.code());
    let status_str = if timed_out {
        "timed_out"
    } else if status.map(|status| status.success()).unwrap_or(false) {
        "completed"
    } else {
        "failed"
    };

    let side_effects = if fs_changes.is_empty() {
        Vec::new()
    } else {
        vec![SideEffectClass::LocalWrite]
    };
    let side_effect_summary = summarize(&fs_changes);
    let artifact_refs = fs_changes
        .iter()
        .map(|change| change.path.clone())
        .collect();

    let receipt_input = CapabilityReceiptInput {
        receipt_id: None,
        action_id: action.action_id.clone(),
        tool_id: action.tool_id.clone(),
        target: action.target.clone(),
        started_at,
        finished_at,
        status: status_str.to_string(),
        input_digest: digest_command(&action.program, &action.args),
        output_digest: sha256_hex(&stdout),
        side_effect_summary,
        side_effects,
        external_ids: Vec::new(),
        artifact_refs,
    };

    Ok(SandboxOutcome {
        receipt_input,
        exit_code,
        timed_out,
        stdout,
        stderr,
        fs_changes,
    })
}

fn drain<R: std::io::Read + Send + 'static>(
    reader: R,
    cap: usize,
) -> JoinHandle<std::io::Result<Vec<u8>>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        // Read at most cap+1 bytes so we never buffer more than one past the cap,
        // then truncate to the cap.
        reader.take(cap as u64 + 1).read_to_end(&mut buffer)?;
        buffer.truncate(cap);
        Ok(buffer)
    })
}

fn join_drain(
    thread: Option<JoinHandle<std::io::Result<Vec<u8>>>>,
) -> Result<Vec<u8>, SandboxError> {
    match thread {
        None => Ok(Vec::new()),
        Some(thread) => match thread.join() {
            Ok(Ok(buffer)) => Ok(buffer),
            Ok(Err(err)) => Err(SandboxError::Io(err)),
            Err(_) => Err(SandboxError::OutputThreadPanicked),
        },
    }
}

/// Snapshot every regular file (and symlink target) under `root` as a map of
/// workspace-relative path to a content digest. Symlinks are recorded by target
/// but never traversed, so the scan cannot escape the workspace.
fn snapshot(root: &Path) -> Result<BTreeMap<String, String>, SandboxError> {
    let mut map = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(SandboxError::Io)? {
            let entry = entry.map_err(SandboxError::Io)?;
            let path = entry.path();
            let file_type = std::fs::symlink_metadata(&path)
                .map_err(SandboxError::Io)?
                .file_type();
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            if file_type.is_symlink() {
                let target = std::fs::read_link(&path)
                    .map(|t| t.to_string_lossy().into_owned())
                    .unwrap_or_default();
                map.insert(rel, format!("symlink:{target}"));
            } else if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                if map.len() >= MAX_WORKSPACE_FILES {
                    return Err(SandboxError::WorkspaceTooLarge {
                        limit: MAX_WORKSPACE_FILES,
                    });
                }
                let hash = hash_file(&path).map_err(SandboxError::Io)?;
                map.insert(rel, hash);
            }
        }
    }
    Ok(map)
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; HASH_CHUNK];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn diff(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) -> Vec<FsChange> {
    let mut changes = Vec::new();
    for (path, hash) in after {
        match before.get(path) {
            None => changes.push(FsChange {
                path: path.clone(),
                kind: FsChangeKind::Created,
            }),
            Some(old) if old != hash => changes.push(FsChange {
                path: path.clone(),
                kind: FsChangeKind::Modified,
            }),
            Some(_) => {}
        }
    }
    for path in before.keys() {
        if !after.contains_key(path) {
            changes.push(FsChange {
                path: path.clone(),
                kind: FsChangeKind::Deleted,
            });
        }
    }
    changes.sort_by(|a, b| a.path.cmp(&b.path));
    changes
}

fn summarize(changes: &[FsChange]) -> String {
    if changes.is_empty() {
        return "no observed filesystem changes".to_string();
    }
    let mut created = 0usize;
    let mut modified = 0usize;
    let mut deleted = 0usize;
    for change in changes {
        match change.kind {
            FsChangeKind::Created => created += 1,
            FsChangeKind::Modified => modified += 1,
            FsChangeKind::Deleted => deleted += 1,
        }
    }
    format!(
        "{} filesystem change(s): {created} created, {modified} modified, {deleted} deleted",
        changes.len()
    )
}

#[derive(Serialize)]
struct CommandDigestView<'a> {
    program: &'a str,
    args: &'a [String],
}

fn digest_command(program: &str, args: &[String]) -> String {
    let view = CommandDigestView { program, args };
    match serde_json::to_vec(&view) {
        Ok(bytes) => sha256_hex(&bytes),
        Err(_) => sha256_hex(program.as_bytes()),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
