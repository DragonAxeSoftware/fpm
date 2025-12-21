use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;

use crate::config::load_manifest;
use crate::git::{GitCliOperations, GitOperations};
use crate::types::{BundleStatus, BUNDLE_DIR};

/// Status entry for display
pub struct StatusEntry {
    pub name: String,
    pub path: String,
    pub status: BundleStatus,
    pub depth: usize,
}

/// Executes the status command with the default GitCliOperations
pub fn execute(manifest_path: &Path) -> Result<()> {
    let git_ops = Arc::new(GitCliOperations::new());
    execute_with_git(manifest_path, git_ops)
}

/// Executes the status command with a custom GitOperations implementation
/// This enables dependency injection for testing
pub fn execute_with_git(manifest_path: &Path, git_ops: Arc<dyn GitOperations>) -> Result<()> {
    let manifest_path = if manifest_path.is_relative() {
        std::env::current_dir()?.join(manifest_path)
    } else {
        manifest_path.to_path_buf()
    };

    println!(
        "{} {}",
        "Bundle status for".cyan(),
        manifest_path.display()
    );
    println!();

    let entries = collect_all_statuses(&manifest_path, git_ops)?;

    // Display status
    if entries.is_empty() {
        println!("{}", "No bundles found.".yellow());
    } else {
        println!(
            "{:<30} {:<10} {}",
            "BUNDLE".bold(),
            "STATUS".bold(),
            "PATH".bold()
        );
        println!("{}", "-".repeat(70));

        for entry in &entries {
            let indent = "  ".repeat(entry.depth);
            let status_colored = match entry.status {
                BundleStatus::Synced => entry.status.to_string().green(),
                BundleStatus::Unsynced => entry.status.to_string().yellow(),
                BundleStatus::Source => entry.status.to_string().blue(),
            };

            println!(
                "{}{:<30} {:<10} {}",
                indent,
                entry.name,
                status_colored,
                entry.path.dimmed()
            );
        }
    }

    // Summary
    println!();
    let synced_count = entries.iter().filter(|e| e.status == BundleStatus::Synced).count();
    let unsynced_count = entries.iter().filter(|e| e.status == BundleStatus::Unsynced).count();
    let source_count = entries.iter().filter(|e| e.status == BundleStatus::Source).count();

    println!(
        "Total: {} synced, {} unsynced, {} source",
        synced_count.to_string().green(),
        unsynced_count.to_string().yellow(),
        source_count.to_string().blue()
    );

    Ok(())
}

/// Collects all bundle statuses without printing (useful for testing)
pub fn collect_all_statuses(
    manifest_path: &Path,
    git_ops: Arc<dyn GitOperations>,
) -> Result<Vec<StatusEntry>> {
    let manifest_path = if manifest_path.is_relative() {
        std::env::current_dir()?.join(manifest_path)
    } else {
        manifest_path.to_path_buf()
    };

    let manifest = load_manifest(&manifest_path)?;
    let parent_dir = manifest_path
        .parent()
        .context("Invalid manifest path")?;

    let mut entries = Vec::new();

    // Check if the current bundle is a source bundle
    if manifest.is_source_bundle() {
        let root_path = parent_dir.join(manifest.root.as_ref().unwrap());
        let status = determine_source_status(git_ops.as_ref(), &root_path)?;
        
        entries.push(StatusEntry {
            name: "(root)".to_string(),
            path: root_path.to_string_lossy().to_string(),
            status,
            depth: 0,
        });
    }

    // Check all bundles in .gitf2 directory
    let bundle_dir = parent_dir.join(BUNDLE_DIR);
    if bundle_dir.exists() {
        collect_bundle_statuses(git_ops.as_ref(), &bundle_dir, 0, &mut entries)?;
    }

    Ok(entries)
}

fn determine_source_status(git_ops: &dyn GitOperations, path: &Path) -> Result<BundleStatus> {
    if !path.exists() {
        return Ok(BundleStatus::Unsynced);
    }

    if !git_ops.is_repository(path) {
        return Ok(BundleStatus::Source);
    }

    if git_ops.has_local_changes(path)? {
        return Ok(BundleStatus::Unsynced);
    }

    Ok(BundleStatus::Source)
}

fn determine_bundle_status(git_ops: &dyn GitOperations, path: &Path) -> Result<BundleStatus> {
    if !path.exists() {
        return Ok(BundleStatus::Unsynced);
    }

    // Check if it has a manifest with root (making it a source)
    let manifest_path = path.join("bundle.toml");
    if manifest_path.exists() {
        if let Ok(manifest) = load_manifest(&manifest_path) {
            if manifest.is_source_bundle() {
                return Ok(BundleStatus::Source);
            }
        }
    }

    if !git_ops.is_repository(path) {
        return Ok(BundleStatus::Unsynced);
    }

    if git_ops.has_local_changes(path)? {
        return Ok(BundleStatus::Unsynced);
    }

    Ok(BundleStatus::Synced)
}

fn collect_bundle_statuses(
    git_ops: &dyn GitOperations,
    bundle_dir: &Path,
    depth: usize,
    entries: &mut Vec<StatusEntry>,
) -> Result<()> {
    if !bundle_dir.exists() {
        return Ok(());
    }

    // Read immediate children only (bundle directories)
    for entry in std::fs::read_dir(bundle_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip hidden directories except .gitf2
        if name.starts_with('.') && name != BUNDLE_DIR {
            continue;
        }

        let status = determine_bundle_status(git_ops, &path)?;
        
        entries.push(StatusEntry {
            name: name.clone(),
            path: path.to_string_lossy().to_string(),
            status,
            depth,
        });

        // Check for nested bundles
        let nested_bundle_dir = path.join(BUNDLE_DIR);
        if nested_bundle_dir.exists() {
            collect_bundle_statuses(git_ops, &nested_bundle_dir, depth + 1, entries)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_status_entry_display() {
        let entry = StatusEntry {
            name: "test-bundle".to_string(),
            path: "/path/to/bundle".to_string(),
            status: BundleStatus::Synced,
            depth: 0,
        };
        
        assert_eq!(entry.name, "test-bundle");
        assert_eq!(entry.status, BundleStatus::Synced);
    }
}
