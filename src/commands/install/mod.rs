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
    let parent_dir = manifest_path
        .parent()
        .context("Invalid manifest path")?;

    // Check for duplicate bundle names
    let bundle_names: Vec<&str> = manifest.bundles.keys().map(|s| s.as_str()).collect();
    let unique_names: HashSet<&str> = bundle_names.iter().copied().collect();
    
    if bundle_names.len() != unique_names.len() {
        anyhow::bail!("Duplicate bundle names detected. Each bundle must have a unique name.");
    }

    let bundle_dir = parent_dir.join(BUNDLE_DIR);

    // Create the .gitf2 directory if it doesn't exist
    if !bundle_dir.exists() {
        fs::create_dir_all(&bundle_dir)
            .with_context(|| format!("Failed to create bundle directory: {}", bundle_dir.display()))?;
    }

    // Check for conflicts before downloading anything
    check_for_conflicts(&manifest.bundles.keys().collect::<Vec<_>>())?;

    for (name, dependency) in &manifest.bundles {
        println!("  {} {}", "Fetching".green(), name);
        
        let target_path = bundle_dir.join(name);
        
        fetch_bundle(git_ops.as_ref(), dependency, &target_path)
            .with_context(|| format!("Failed to fetch bundle: {}", name))?;
        
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
    let parent_dir = manifest_path
        .parent()
        .context("Invalid manifest path")?;
    
    let bundle_dir = parent_dir.join(BUNDLE_DIR);
    
    if !bundle_dir.exists() {
        fs::create_dir_all(&bundle_dir)?;
    }
    
    for (name, dependency) in &manifest.bundles {
        println!("    {} (nested) {}", "Fetching".blue(), name);
        
        let target_path = bundle_dir.join(name);
        fetch_bundle(git_ops.as_ref(), dependency, &target_path)?;
        
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

    #[test]
    fn test_check_for_conflicts_no_conflicts() {
        let bundle_a = "bundle-a".to_string();
        let bundle_b = "bundle-b".to_string();
        let bundle_c = "bundle-c".to_string();
        let names = vec![&bundle_a, &bundle_b, &bundle_c];
        let result = check_for_conflicts(&names);
        assert!(result.is_ok());
    }
}
