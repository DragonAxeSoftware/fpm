use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;

use crate::config::load_manifest;
use crate::git::{init_bundle_for_publish, GitCliOperations, GitOperations};
use crate::types::{DEFAULT_BRANCH, DEFAULT_REMOTE};

/// Executes the publish command with the default GitCliOperations
pub fn execute(manifest_path: &Path) -> Result<()> {
    let git_ops = Arc::new(GitCliOperations::new());
    execute_with_git(manifest_path, git_ops)
}

/// Executes the publish command with a custom GitOperations implementation
/// This enables dependency injection for testing
pub fn execute_with_git(manifest_path: &Path, git_ops: Arc<dyn GitOperations>) -> Result<()> {
    let manifest_path = if manifest_path.is_relative() {
        std::env::current_dir()?.join(manifest_path)
    } else {
        manifest_path.to_path_buf()
    };

    println!(
        "{} {}",
        "Publishing bundles from".cyan(),
        manifest_path.display()
    );

    let manifest = load_manifest(&manifest_path)?;
    let parent_dir = manifest_path.parent().context("Invalid manifest path")?;

    // Check if this is a source bundle
    if manifest.root.is_none() {
        println!(
            "{}",
            "This bundle.toml has no 'root' defined. Nothing to publish.".yellow()
        );
        return Ok(());
    }

    let root_dir = parent_dir.join(manifest.root.as_ref().unwrap());

    if !root_dir.exists() {
        anyhow::bail!(
            "Root directory '{}' does not exist. Cannot publish.",
            root_dir.display()
        );
    }

    // Check for changes
    if git_ops.is_repository(&root_dir) && !git_ops.has_local_changes(&root_dir)? {
        println!("{}", "No changes to publish.".yellow());
        return Ok(());
    }

    // Find the remote URL from bundles (self-reference pattern)
    // For a source bundle to be publishable, we need to know where to push
    // This could be stored in a separate field or inferred
    let remote_url = get_publish_remote(&manifest_path, git_ops.as_ref())?;

    publish_bundle(
        git_ops.as_ref(),
        &root_dir,
        &remote_url,
        &manifest.fpm_version,
    )?;

    println!("{}", "Published successfully!".green().bold());
    Ok(())
}

fn get_publish_remote(manifest_path: &Path, git_ops: &dyn GitOperations) -> Result<String> {
    // Try to read the remote from git config if already initialized
    let parent = manifest_path.parent().context("Invalid manifest path")?;

    if git_ops.is_repository(parent) {
        // Try to get the fpm remote URL
        if let Ok(repo) = git2::Repository::open(parent) {
            if let Ok(remote) = repo.find_remote(DEFAULT_REMOTE) {
                if let Some(url) = remote.url() {
                    return Ok(url.to_string());
                }
            }
            // Fall back to origin
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    return Ok(url.to_string());
                }
            }
        }
    }

    anyhow::bail!(
        "No remote URL configured for publishing. \
        Please initialize the bundle with a git remote or add a 'publish_url' field."
    )
}

fn publish_bundle(
    git_ops: &dyn GitOperations,
    root_dir: &Path,
    remote_url: &str,
    version: &str,
) -> Result<()> {
    println!("  {} {}", "Publishing".green(), root_dir.display());

    // Initialize git if needed
    init_bundle_for_publish(git_ops, root_dir, remote_url)?;

    // Commit all changes
    let commit_message = format!("fpm publish v{}", version);
    git_ops.commit_all(root_dir, &commit_message)?;

    // Push to remote
    git_ops.push(root_dir, DEFAULT_REMOTE, DEFAULT_BRANCH)?;

    println!("  {} v{}", "✓ Published".green(), version);
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    // Tests would require mocking file system and git operations
    // For now, integration tests would be more appropriate
}
