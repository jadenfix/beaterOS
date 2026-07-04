//! End-to-end tests for the scoped shell lane. Each test builds a real
//! temporary workspace, runs a real process, and asserts on the *observed*
//! receipt — proving the lane records ground truth, not claims.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

use beater_os_core::{CapabilitySelector, ReceiptLedger, ResourceKind, SideEffectClass};
use beater_os_sandbox::{FsChangeKind, SandboxError, ScopedShellAction, run};

type TestResult = Result<(), Box<dyn Error>>;

/// A unique, freshly-created temp workspace for one test. Cleaned up by the
/// caller via `cleanup`.
fn workspace(name: &str) -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("beater-sandbox-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn target() -> CapabilitySelector {
    CapabilitySelector {
        resource_kind: ResourceKind::Workspace,
        resource_id: "ws-1".to_string(),
    }
}

fn sh(ws: &Path, script: &str) -> ScopedShellAction {
    ScopedShellAction::new("A1", "tool:shell", target(), ws.to_path_buf(), "/bin/sh")
        .arg("-c")
        .arg(script)
}

#[test]
fn observes_created_file_as_side_effect() -> TestResult {
    let ws = workspace("created")?;
    let outcome = run(&sh(&ws, "printf hi > out.txt"))?;

    assert_eq!(outcome.receipt_input.status, "completed");
    assert_eq!(outcome.exit_code, Some(0));
    assert_eq!(outcome.fs_changes.len(), 1);
    assert_eq!(outcome.fs_changes[0].path, "out.txt");
    assert_eq!(outcome.fs_changes[0].kind, FsChangeKind::Created);
    assert_eq!(
        outcome.receipt_input.side_effects,
        vec![SideEffectClass::LocalWrite]
    );
    assert!(
        outcome
            .receipt_input
            .side_effect_summary
            .contains("1 created")
    );
    assert_eq!(
        outcome.receipt_input.artifact_refs,
        vec!["out.txt".to_string()]
    );

    // The observed receipt must be a valid, chainable receipt.
    let mut ledger = ReceiptLedger::new();
    let receipt = ledger.append(outcome.receipt_input)?;
    assert_eq!(receipt.status, "completed");
    ledger.verify_chain()?;

    cleanup(&ws);
    Ok(())
}

#[test]
fn no_filesystem_change_yields_no_side_effect() -> TestResult {
    let ws = workspace("nochange")?;
    let outcome = run(&sh(&ws, "printf hello"))?; // stdout only, no fs write

    assert_eq!(outcome.receipt_input.status, "completed");
    assert!(outcome.fs_changes.is_empty());
    assert!(outcome.receipt_input.side_effects.is_empty());
    assert_eq!(
        outcome.receipt_input.side_effect_summary,
        "no observed filesystem changes"
    );
    // stdout was captured and digested.
    assert_eq!(outcome.stdout, b"hello");

    cleanup(&ws);
    Ok(())
}

#[test]
fn inherited_environment_is_not_visible_to_the_child() -> TestResult {
    let ws = workspace("env")?;
    // `/usr/bin/env` prints exactly the child environment with no shell-injected
    // defaults. With env_clear + one explicit var, only that var may appear, and
    // an always-present parent var like PATH must NOT.
    let action = ScopedShellAction::new("A1", "tool:shell", target(), ws.clone(), "/usr/bin/env")
        .env_var("MYVAR", "present");
    let outcome = run(&action)?;
    let stdout = String::from_utf8_lossy(&outcome.stdout);

    assert!(
        stdout.contains("MYVAR=present"),
        "explicit env var should pass through, got: {stdout:?}"
    );
    assert!(
        !stdout.contains("PATH="),
        "inherited PATH must not leak into the child, got: {stdout:?}"
    );

    cleanup(&ws);
    Ok(())
}

#[test]
fn missing_workspace_fails_closed() -> TestResult {
    let ws = std::env::temp_dir().join(format!("beater-sandbox-{}-absent", std::process::id()));
    let _ = std::fs::remove_dir_all(&ws);
    let action = sh(&ws, "true");
    match run(&action) {
        Err(SandboxError::WorkspaceMissing(_)) => Ok(()),
        other => Err(format!("expected WorkspaceMissing, got {other:?}").into()),
    }
}

#[test]
fn file_as_workspace_fails_closed() -> TestResult {
    let ws = workspace("filews")?;
    let file = ws.join("not-a-dir");
    std::fs::write(&file, b"x")?;
    let action = sh(&file, "true");
    let result = run(&action);
    cleanup(&ws);
    match result {
        Err(SandboxError::WorkspaceNotDir(_)) => Ok(()),
        other => Err(format!("expected WorkspaceNotDir, got {other:?}").into()),
    }
}

#[test]
fn nonzero_exit_is_reported_as_failed() -> TestResult {
    let ws = workspace("fail")?;
    let outcome = run(&sh(&ws, "exit 3"))?;
    assert_eq!(outcome.receipt_input.status, "failed");
    assert_eq!(outcome.exit_code, Some(3));
    assert!(!outcome.timed_out);
    assert!(outcome.fs_changes.is_empty());
    cleanup(&ws);
    Ok(())
}

#[test]
fn timeout_kills_the_process_and_reports_it() -> TestResult {
    let ws = workspace("timeout")?;
    let action = sh(&ws, "sleep 5").with_timeout(Duration::from_millis(150));
    let outcome = run(&action)?;
    assert!(outcome.timed_out, "expected the run to time out");
    assert_eq!(outcome.receipt_input.status, "timed_out");
    assert_eq!(outcome.exit_code, None);
    cleanup(&ws);
    Ok(())
}

#[test]
fn modification_and_deletion_are_observed() -> TestResult {
    let ws = workspace("modify")?;
    std::fs::write(ws.join("a.txt"), b"old")?;
    std::fs::write(ws.join("b.txt"), b"doomed")?;
    let outcome = run(&sh(&ws, "printf new > a.txt; rm b.txt"))?;

    assert_eq!(outcome.receipt_input.status, "completed");
    let mut by_path: Vec<(String, FsChangeKind)> = outcome
        .fs_changes
        .iter()
        .map(|c| (c.path.clone(), c.kind))
        .collect();
    by_path.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        by_path,
        vec![
            ("a.txt".to_string(), FsChangeKind::Modified),
            ("b.txt".to_string(), FsChangeKind::Deleted),
        ]
    );
    assert_eq!(
        outcome.receipt_input.side_effects,
        vec![SideEffectClass::LocalWrite]
    );

    cleanup(&ws);
    Ok(())
}
