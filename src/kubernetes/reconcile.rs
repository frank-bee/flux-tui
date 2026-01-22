//! Flux CLI wrapper for reconciliation operations
//!
//! Uses the `flux` CLI for reconciliation operations because:
//! 1. It handles the complexity of Flux's reconciliation logic
//! 2. It properly manages source dependencies
//! 3. It provides consistent behavior with the official tooling

use color_eyre::{eyre::eyre, Result};
use std::process::Command;

/// Reconcile a Flux resource using the flux CLI
///
/// # Arguments
/// * `name` - Resource name
/// * `namespace` - Resource namespace
/// * `kind` - Resource kind (kustomization, helmrelease, helmchart)
/// * `with_source` - Whether to reconcile the source first
pub async fn reconcile(name: &str, namespace: &str, kind: &str, with_source: bool) -> Result<()> {
    let mut args = vec!["reconcile", kind, name, "-n", namespace];

    if with_source {
        args.push("--with-source");
    }

    run_flux_command(&args).await
}

/// Toggle suspend status on a Flux resource
///
/// # Arguments
/// * `name` - Resource name
/// * `namespace` - Resource namespace
/// * `kind` - Resource kind (kustomization, helmrelease)
/// * `is_currently_suspended` - Current suspend status
pub async fn toggle_suspend(
    name: &str,
    namespace: &str,
    kind: &str,
    is_currently_suspended: bool,
) -> Result<()> {
    let action = if is_currently_suspended {
        "resume"
    } else {
        "suspend"
    };

    let args = vec![action, kind, name, "-n", namespace];

    run_flux_command(&args).await
}

/// Run a flux CLI command
async fn run_flux_command(args: &[&str]) -> Result<()> {
    // Spawn the command in a blocking task to not block the async runtime
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    tokio::task::spawn_blocking(move || {
        let output = Command::new("flux")
            .args(&args)
            .output()
            .map_err(|e| eyre!("Failed to execute flux command: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(eyre!(
                "Flux command failed: {}\n{}",
                stderr.trim(),
                stdout.trim()
            ))
        }
    })
    .await?
}

/// Check if the flux CLI is available
#[allow(dead_code)]
pub fn is_flux_available() -> bool {
    Command::new("flux")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
