use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;

use crate::config::{load_manifest, save_manifest};
use crate::git::{GitCliOperations, GitOperations};
use crate::types::{BundleManifest, BUNDLE_DIR, DEFAULT_BRANCH};

/// Executes the push command with the default GitCliOperations
pub fn execute(
    manifest_path: &Path,
    bundle_name: Option<&str>,
    message: Option<&str>,
) -> Result<()> {
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
    let parent_dir = manifest_path.parent().context("Invalid manifest path")?;
    let bundle_dir = parent_dir.join(BUNDLE_DIR);

    if !bundle_dir.exists() {
        anyhow::bail!("No bundles installed. Run 'fpm install' first.");
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
            println!("  {} {} (not installed)", "Skipping".yellow(), name);
            stats.skipped += 1;
            continue;
        }

        if !git_ops.is_repository(&bundle_path) {
            println!("  {} {} (not a git repository)", "Skipping".yellow(), name);
            stats.skipped += 1;
            continue;
        }

        // Push this bundle and all its nested bundles recursively
        push_bundle_recursive(
            git_ops.as_ref(),
            &name,
            &bundle_path,
            message,
            0,
            &mut stats,
        );
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

            for nested_name in nested_manifest.bundles.keys() {
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
                println!("{}{} {}: {}", indent, "Failed".red(), name, e);
                stats.errors += 1;
            }
        }
    }
}

enum PushResult {
    Pushed,
    NoChanges,
}

/// Bump patch version (0.0.1 -> 0.0.2)
fn bump_patch_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() == 3 {
        if let Ok(patch) = parts[2].parse::<u32>() {
            return format!("{}.{}.{}", parts[0], parts[1], patch + 1);
        }
    }
    version.to_string()
}

/// Check if the version was manually changed by comparing working tree to HEAD
fn version_was_changed(git_ops: &dyn GitOperations, bundle_path: &Path) -> Result<bool> {
    let manifest_path = bundle_path.join("bundle.toml");

    // Get the committed version from HEAD
    let committed_content = git_ops.get_file_from_head(bundle_path, "bundle.toml")?;
    let committed_manifest: BundleManifest =
        toml::from_str(&committed_content).context("Failed to parse committed bundle.toml")?;

    // Get the current version from working tree
    let current_content =
        std::fs::read_to_string(&manifest_path).context("Failed to read bundle.toml")?;
    let current_manifest: BundleManifest =
        toml::from_str(&current_content).context("Failed to parse bundle.toml")?;

    Ok(committed_manifest.version != current_manifest.version)
}

/// Auto-increment the version in the manifest if it hasn't been manually changed
fn auto_increment_version_if_needed(
    git_ops: &dyn GitOperations,
    bundle_path: &Path,
    indent: &str,
) -> Result<()> {
    let manifest_path = bundle_path.join("bundle.toml");

    // Check if version was already changed manually
    match version_was_changed(git_ops, bundle_path) {
        Ok(true) => {
            // Version was manually changed, nothing to do
            return Ok(());
        }
        Ok(false) => {
            // Version not changed, we need to auto-increment
        }
        Err(_) => {
            // Could not compare (maybe no HEAD commit yet), skip auto-increment
            return Ok(());
        }
    }

    // Load manifest, bump version, save
    let content = std::fs::read_to_string(&manifest_path)?;
    let mut manifest: BundleManifest =
        toml::from_str(&content).context("Failed to parse bundle.toml")?;

    let old_version = manifest
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".to_string());
    let new_version = bump_patch_version(&old_version);
    manifest.version = Some(new_version.clone());

    save_manifest(&manifest, &manifest_path)?;

    println!(
        "{}Auto-incremented version: {} -> {}",
        indent,
        old_version.yellow(),
        new_version.green()
    );

    Ok(())
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
        println!("{}{} {} (no changes)", indent, "Skipping".cyan(), name);
        return Ok(PushResult::NoChanges);
    }

    println!("{}{} {}", indent, "Pushing".green(), name);

    // Auto-increment version if user forgot to change it
    auto_increment_version_if_needed(git_ops, bundle_path, indent)?;

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
        println!("{} {} bundle(s)", "Pushed".green().bold(), stats.pushed);
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
        println!("{} No bundles had changes to push.", "Note:".cyan());
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bump_patch_version() {
        assert_eq!(bump_patch_version("0.0.1"), "0.0.2");
        assert_eq!(bump_patch_version("1.0.0"), "1.0.1");
        assert_eq!(bump_patch_version("1.2.3"), "1.2.4");
        assert_eq!(bump_patch_version("0.0.99"), "0.0.100");
        // Invalid versions pass through unchanged
        assert_eq!(bump_patch_version("invalid"), "invalid");
        assert_eq!(bump_patch_version("1.0"), "1.0");
    }
}
