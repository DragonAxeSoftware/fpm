use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;

use crate::config::load_manifest;
use crate::git::{GitCliOperations, GitOperations};
use crate::types::{BUNDLE_DIR, DEFAULT_BRANCH};

/// Executes the push command with the default GitCliOperations
pub fn execute(manifest_path: &Path, bundle_name: Option<&str>, message: Option<&str>) -> Result<()> {
    let git_ops = Arc::new(GitCliOperations::new());
    execute_with_git(manifest_path, bundle_name, message, git_ops)
}

/// Executes the push command with a custom GitOperations implementation
/// This enables dependency injection for testing
pub fn execute_with_git(
    manifest_path: &Path,
    bundle_name: Option<&str>,
    message: Option<&str>,
    git_ops: Arc<dyn GitOperations>,
) -> Result<()> {
    let manifest_path = if manifest_path.is_relative() {
        std::env::current_dir()?.join(manifest_path)
    } else {
        manifest_path.to_path_buf()
    };

    let manifest = load_manifest(&manifest_path)?;
    let parent_dir = manifest_path
        .parent()
        .context("Invalid manifest path")?;
    let bundle_dir = parent_dir.join(BUNDLE_DIR);

    if !bundle_dir.exists() {
        anyhow::bail!(
            "No bundles installed. Run 'fpm install' first."
        );
    }

    // Determine which bundles to push
    let bundles_to_push: Vec<String> = if let Some(name) = bundle_name {
        // Push specific bundle
        if !manifest.bundles.contains_key(name) {
            anyhow::bail!(
                "Bundle '{}' not found in manifest. Available bundles: {:?}",
                name,
                manifest.bundles.keys().collect::<Vec<_>>()
            );
        }
        vec![name.to_string()]
    } else {
        // Push all bundles with changes
        manifest.bundles.keys().cloned().collect()
    };

    let mut stats = PushStats::default();

    for name in bundles_to_push {
        let bundle_path = bundle_dir.join(&name);
        
        if !bundle_path.exists() {
            println!(
                "  {} {} (not installed)",
                "Skipping".yellow(),
                name
            );
            stats.skipped += 1;
            continue;
        }

        if !git_ops.is_repository(&bundle_path) {
            println!(
                "  {} {} (not a git repository)",
                "Skipping".yellow(),
                name
            );
            stats.skipped += 1;
            continue;
        }

        // Push this bundle and all its nested bundles recursively
        push_bundle_recursive(git_ops.as_ref(), &name, &bundle_path, message, 0, &mut stats);
    }

    print_summary(&stats);

    Ok(())
}

#[derive(Default)]
struct PushStats {
    pushed: u32,
    skipped: u32,
    auth_failed: u32,
    errors: u32,
}

/// Recursively push a bundle and all its nested bundles
fn push_bundle_recursive(
    git_ops: &dyn GitOperations,
    name: &str,
    bundle_path: &Path,
    message: Option<&str>,
    depth: usize,
    stats: &mut PushStats,
) {
    let indent = "  ".repeat(depth + 1);
    
    // First, check for and push nested bundles
    let nested_manifest_path = bundle_path.join("bundle.toml");
    if nested_manifest_path.exists() {
        if let Ok(nested_manifest) = crate::config::load_manifest(&nested_manifest_path) {
            let nested_bundle_dir = bundle_path.join(BUNDLE_DIR);
            
            for (nested_name, _) in &nested_manifest.bundles {
                let nested_path = nested_bundle_dir.join(nested_name);
                
                if nested_path.exists() && git_ops.is_repository(&nested_path) {
                    push_bundle_recursive(
                        git_ops,
                        nested_name,
                        &nested_path,
                        message,
                        depth + 1,
                        stats,
                    );
                }
            }
        }
    }

    // Now push this bundle
    match push_single_bundle(git_ops, name, bundle_path, message, &indent) {
        Ok(PushResult::Pushed) => stats.pushed += 1,
        Ok(PushResult::NoChanges) => stats.skipped += 1,
        Err(e) => {
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("permission denied")
                || error_msg.contains("authentication")
                || error_msg.contains("403")
                || error_msg.contains("401")
                || error_msg.contains("could not read from remote")
            {
                println!(
                    "{}⚠ {} {} (no push access - local changes preserved)",
                    indent,
                    "Warning:".yellow().bold(),
                    name
                );
                stats.auth_failed += 1;
            } else {
                println!(
                    "{}{} {}: {}",
                    indent,
                    "Failed".red(),
                    name,
                    e
                );
                stats.errors += 1;
            }
        }
    }
}

enum PushResult {
    Pushed,
    NoChanges,
}

/// Push a single bundle's changes to its remote
fn push_single_bundle(
    git_ops: &dyn GitOperations,
    name: &str,
    bundle_path: &Path,
    message: Option<&str>,
    indent: &str,
) -> Result<PushResult> {
    // Check for local changes
    if !git_ops.has_local_changes(bundle_path)? {
        println!(
            "{}{} {} (no changes)",
            indent,
            "Skipping".cyan(),
            name
        );
        return Ok(PushResult::NoChanges);
    }

    println!("{}{} {}", indent, "Pushing".green(), name);

    // Commit all changes
    let commit_msg = message.unwrap_or("fpm push: Update bundle");
    git_ops.commit_all(bundle_path, commit_msg)?;

    // Push to origin (the cloned remote)
    git_ops.push(bundle_path, "origin", DEFAULT_BRANCH)?;

    println!("{}{} {}", indent, "✓".green(), name);
    Ok(PushResult::Pushed)
}

fn print_summary(stats: &PushStats) {
    println!();
    
    if stats.pushed > 0 {
        println!(
            "{} {} bundle(s)",
            "Pushed".green().bold(),
            stats.pushed
        );
    }
    
    if stats.auth_failed > 0 {
        println!(
            "{} {} bundle(s) have local changes but no push access",
            "Warning:".yellow().bold(),
            stats.auth_failed
        );
    }
    
    if stats.errors > 0 {
        println!(
            "{} {} bundle(s) failed to push",
            "Error:".red().bold(),
            stats.errors
        );
    }
    
    if stats.pushed == 0 && stats.auth_failed == 0 && stats.errors == 0 {
        println!(
            "{} No bundles had changes to push.",
            "Note:".cyan()
        );
    }
}

#[cfg(test)]
mod unit_tests {
    // Integration tests are more appropriate for this command
    // as it requires real git operations
}
