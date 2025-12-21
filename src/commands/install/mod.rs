use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::config::load_manifest;
use crate::git::{fetch_bundle, GitCliOperations, GitOperations};
use crate::types::BUNDLE_DIR;

/// Executes the install command with the default GitCliOperations
pub fn execute(manifest_path: &Path) -> Result<()> {
    let git_ops = Arc::new(GitCliOperations::new());
    execute_with_git(manifest_path, git_ops)
}

/// Ensures the bundle's .gitignore contains an entry for the .fpm directory
/// This prevents nested bundle directories from being pushed to source repos
fn ensure_fpm_in_gitignore(bundle_path: &Path) -> Result<()> {
    let gitignore_path = bundle_path.join(".gitignore");
    let fpm_entry = format!("{}/", BUNDLE_DIR);

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        // Check if .fpm/ is already in gitignore (with or without trailing slash)
        let has_fpm_ignore = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == BUNDLE_DIR
                || trimmed == fpm_entry
                || trimmed == format!("/{}", BUNDLE_DIR)
                || trimmed == format!("/{}/", BUNDLE_DIR)
        });

        if !has_fpm_ignore {
            // Append .fpm/ to existing gitignore
            let new_content = if content.ends_with('\n') {
                format!("{}{}\n", content, fpm_entry)
            } else {
                format!("{}\n{}\n", content, fpm_entry)
            };
            fs::write(&gitignore_path, new_content)?;
        }
    } else {
        // Create new gitignore with .fpm/
        fs::write(&gitignore_path, format!("{}\n", fpm_entry))?;
    }

    Ok(())
}

/// Executes the install command with a custom GitOperations implementation
/// This enables dependency injection for testing
pub fn execute_with_git(manifest_path: &Path, git_ops: Arc<dyn GitOperations>) -> Result<()> {
    let manifest_path = if manifest_path.is_relative() {
        std::env::current_dir()?.join(manifest_path)
    } else {
        manifest_path.to_path_buf()
    };

    println!(
        "{} {}",
        "Installing bundles from".cyan(),
        manifest_path.display()
    );

    let manifest = load_manifest(&manifest_path)?;
    let parent_dir = manifest_path.parent().context("Invalid manifest path")?;

    // Check for duplicate bundle names
    let bundle_names: Vec<&str> = manifest.bundles.keys().map(|s| s.as_str()).collect();
    let unique_names: HashSet<&str> = bundle_names.iter().copied().collect();

    if bundle_names.len() != unique_names.len() {
        anyhow::bail!("Duplicate bundle names detected. Each bundle must have a unique name.");
    }

    let bundle_dir = parent_dir.join(BUNDLE_DIR);

    // Create the .fpm directory if it doesn't exist
    if !bundle_dir.exists() {
        fs::create_dir_all(&bundle_dir).with_context(|| {
            format!(
                "Failed to create bundle directory: {}",
                bundle_dir.display()
            )
        })?;
    }

    // Check for conflicts before downloading anything
    check_for_conflicts(&manifest.bundles.keys().collect::<Vec<_>>())?;

    for (name, dependency) in &manifest.bundles {
        println!("  {} {}", "Fetching".green(), name);

        let target_path = bundle_dir.join(name);

        fetch_bundle(git_ops.as_ref(), dependency, &target_path)
            .with_context(|| format!("Failed to fetch bundle: {}", name))?;

        // Ensure .fpm is in the bundle's .gitignore to prevent nested bundles
        // from being pushed to source repositories
        ensure_fpm_in_gitignore(&target_path)?;

        // Handle nested bundles recursively
        let nested_manifest_path = target_path.join("bundle.toml");
        if nested_manifest_path.exists() {
            install_nested_bundles(&nested_manifest_path, git_ops.clone())?;
        }

        println!("  {} {}", "✓".green(), name);
    }

    println!("{}", "All bundles installed successfully!".green().bold());
    Ok(())
}

fn check_for_conflicts(names: &[&String]) -> Result<()> {
    let mut seen = HashSet::new();

    for name in names {
        if !seen.insert(*name) {
            anyhow::bail!(
                "Conflict detected: bundle '{}' appears multiple times. \
                Each bundle must have a unique name.",
                name
            );
        }
    }

    Ok(())
}

fn install_nested_bundles(manifest_path: &Path, git_ops: Arc<dyn GitOperations>) -> Result<()> {
    let manifest = load_manifest(manifest_path)?;
    let parent_dir = manifest_path.parent().context("Invalid manifest path")?;

    let bundle_dir = parent_dir.join(BUNDLE_DIR);

    if !bundle_dir.exists() {
        fs::create_dir_all(&bundle_dir)?;
    }

    for (name, dependency) in &manifest.bundles {
        println!("    {} (nested) {}", "Fetching".blue(), name);

        let target_path = bundle_dir.join(name);
        fetch_bundle(git_ops.as_ref(), dependency, &target_path)?;

        // Ensure .fpm is in the bundle's .gitignore
        ensure_fpm_in_gitignore(&target_path)?;

        // Recursive nested bundles
        let nested_manifest_path = target_path.join("bundle.toml");
        if nested_manifest_path.exists() {
            install_nested_bundles(&nested_manifest_path, git_ops.clone())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_for_conflicts_no_conflicts() {
        let bundle_a = "bundle-a".to_string();
        let bundle_b = "bundle-b".to_string();
        let bundle_c = "bundle-c".to_string();
        let names = vec![&bundle_a, &bundle_b, &bundle_c];
        let result = check_for_conflicts(&names);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_fpm_in_gitignore_creates_new() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path();

        // No .gitignore exists
        assert!(!bundle_path.join(".gitignore").exists());

        ensure_fpm_in_gitignore(bundle_path).unwrap();

        let content = fs::read_to_string(bundle_path.join(".gitignore")).unwrap();
        assert!(content.contains(".fpm/"));
    }

    #[test]
    fn test_ensure_fpm_in_gitignore_appends_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path();

        // Create existing .gitignore without .fpm
        fs::write(bundle_path.join(".gitignore"), "/target\n*.log\n").unwrap();

        ensure_fpm_in_gitignore(bundle_path).unwrap();

        let content = fs::read_to_string(bundle_path.join(".gitignore")).unwrap();
        assert!(content.contains("/target"));
        assert!(content.contains("*.log"));
        assert!(content.contains(".fpm/"));
    }

    #[test]
    fn test_ensure_fpm_in_gitignore_skips_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path();

        // Create existing .gitignore with .fpm already
        fs::write(bundle_path.join(".gitignore"), "/target\n.fpm/\n").unwrap();

        ensure_fpm_in_gitignore(bundle_path).unwrap();

        let content = fs::read_to_string(bundle_path.join(".gitignore")).unwrap();
        // Should only have one .fpm/ entry, not duplicated
        assert_eq!(content.matches(".fpm").count(), 1);
    }

    #[test]
    fn test_ensure_fpm_in_gitignore_recognizes_variants() {
        let temp_dir = TempDir::new().unwrap();

        // Test with ".fpm" (no trailing slash)
        let bundle_path1 = temp_dir.path().join("bundle1");
        fs::create_dir(&bundle_path1).unwrap();
        fs::write(bundle_path1.join(".gitignore"), ".fpm\n").unwrap();
        ensure_fpm_in_gitignore(&bundle_path1).unwrap();
        let content1 = fs::read_to_string(bundle_path1.join(".gitignore")).unwrap();
        assert_eq!(content1.matches(".fpm").count(), 1);

        // Test with "/.fpm/" (leading slash)
        let bundle_path2 = temp_dir.path().join("bundle2");
        fs::create_dir(&bundle_path2).unwrap();
        fs::write(bundle_path2.join(".gitignore"), "/.fpm/\n").unwrap();
        ensure_fpm_in_gitignore(&bundle_path2).unwrap();
        let content2 = fs::read_to_string(bundle_path2.join(".gitignore")).unwrap();
        assert_eq!(content2.matches(".fpm").count(), 1);
    }
}
