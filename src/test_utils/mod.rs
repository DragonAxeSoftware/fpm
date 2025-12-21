//! Common test utilities shared between unit tests and integration tests
//!
//! This module contains helpers for:
//! - Test directory management
//! - Project structure creation
//! - Bundle manifest creation

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::save_manifest;
use crate::types::{BundleDependency, BundleManifest, FPM_IDENTIFIER};

/// Gets the test directory path for a given test category
pub fn get_test_dir(category: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tests")
        .join(category)
}

/// Sets up a clean test environment
pub fn setup_test_env(category: &str, test_name: &str) -> Result<PathBuf> {
    let test_dir = get_test_dir(category).join(test_name);

    // Clean up previous test run
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }

    fs::create_dir_all(&test_dir)?;
    Ok(test_dir)
}

/// Cleans up test environment
pub fn cleanup_test_env(category: &str, test_name: &str) -> Result<()> {
    let test_dir = get_test_dir(category).join(test_name);
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    Ok(())
}

/// Creates a sample project structure with non-fpm files
pub fn create_sample_project(base_dir: &Path) -> Result<()> {
    // Create typical project structure
    let src_dir = base_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create some sample files
    fs::write(base_dir.join("README.md"), "# My Project\n\nA sample project.")?;
    fs::write(
        src_dir.join("main.rs"),
        "fn main() {\n    println!(\"Hello!\");\n}",
    )?;
    fs::write(base_dir.join(".gitignore"), "/target\n.fpm/")?;

    // Create a design directory where we'll add bundles
    let design_dir = src_dir.join("design");
    fs::create_dir_all(&design_dir)?;
    fs::write(design_dir.join("styles.css"), "body { margin: 0; }")?;

    Ok(())
}

/// Creates a bundle.toml manifest in the specified directory
pub fn create_bundle_manifest(
    dir: &Path,
    description: Option<&str>,
    root: Option<&str>,
    bundles: HashMap<String, BundleDependency>,
) -> Result<PathBuf> {
    let manifest = BundleManifest {
        fpm_version: "0.1.0".to_string(),
        identifier: FPM_IDENTIFIER.to_string(),
        name: None,
        version: None,
        description: description.map(String::from),
        root: root.map(PathBuf::from),
        bundles,
    };

    let manifest_path = dir.join("bundle.toml");
    save_manifest(&manifest, &manifest_path)?;

    Ok(manifest_path)
}

/// Checks if git is installed and available in PATH
pub fn is_git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Gets the path to the fpm binary
pub fn get_fpm_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    
    #[cfg(windows)]
    path.push("fpm.exe");
    
    #[cfg(not(windows))]
    path.push("fpm");
    
    path
}

/// Runs the fpm binary with the given arguments
pub fn run_fpm(args: &[&str], working_dir: &Path) -> Result<std::process::Output> {
    let binary_path = get_fpm_binary_path();
    
    if !binary_path.exists() {
        anyhow::bail!(
            "fpm binary not found at {:?}. Run 'cargo build' first.",
            binary_path
        );
    }
    
    let output = std::process::Command::new(&binary_path)
        .args(args)
        .current_dir(working_dir)
        .output()?;
    
    Ok(output)
}
