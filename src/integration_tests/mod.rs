//! Integration tests for gitf2
//!
//! These tests run with real external dependencies (actual git repositories).
//! They require:
//! - Git installed and available in PATH
//! - Network access to the test repositories
//!
//! Run with: `cargo test integration_tests -- --ignored`

use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use crate::test_utils::{
    create_bundle_manifest, create_sample_project, get_gitf2_binary_path,
    is_git_available, run_gitf2, setup_test_env,
    // cleanup_test_env, // Commented out to inspect results in .tests folder
};
use crate::types::{BundleDependency, BUNDLE_DIR};

const TEST_CATEGORY: &str = "integration";

/// The HTTPS URL of the test repository (for public access without SSH key)
const EXAMPLE_BUNDLE_REPO_HTTPS: &str = "https://github.com/DragonAxeSoftware/gitf2-example-1.git";

/// The SSH URL of the test repository (for SSH authentication)
/// NOTE: SSH authentication is not fully implemented in tests yet.
/// TODO: Load SSH key path from environment variable or .env file when implementing SSH tests.
#[allow(dead_code)]
const EXAMPLE_BUNDLE_REPO_SSH: &str = "git@github.com:DragonAxeSoftware/gitf2-example-1.git";

/// Checks preconditions before running integration tests
fn check_preconditions() -> Result<()> {
    if !is_git_available() {
        anyhow::bail!(
            "Git is not installed or not in PATH. \
            Please install git or ensure it's correctly configured in your PATH environment variable."
        );
    }

    let binary_path = get_gitf2_binary_path();
    if !binary_path.exists() {
        anyhow::bail!(
            "gitf2 binary not found at {:?}. \
            Please run 'cargo build' or use the build script at scripts/devops/build.ps1",
            binary_path
        );
    }

    Ok(())
}

#[test]
#[ignore] // Run only when explicitly requested: cargo test integration_tests -- --ignored
fn test_install_from_real_git_repository() -> Result<()> {
    // Check preconditions
    check_preconditions()?;

    let test_name = "install_real_git";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    // Step 1: Create a sample project structure
    create_sample_project(&test_dir)?;

    // Step 2: Create a bundle.toml that references a real git repository via HTTPS
    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    bundles.insert(
        "ui-assets".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_BUNDLE_REPO_HTTPS.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
        },
    );

    let _manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Design assets from real git repository"),
        None,
        bundles,
    )?;

    // Step 3: Run gitf2 install command using the real binary
    println!("Running gitf2 install in {:?}", design_dir);
    let output = run_gitf2(&["install"], &design_dir)?;

    // Print output for debugging
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Step 4: Verify the command succeeded
    assert!(
        output.status.success(),
        "gitf2 install should succeed. Exit code: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 5: Verify the bundle was installed
    let bundle_dir = design_dir.join(BUNDLE_DIR);
    assert!(bundle_dir.exists(), "Bundle directory should exist");

    let installed_bundle = bundle_dir.join("ui-assets");
    assert!(
        installed_bundle.exists(),
        "ui-assets bundle should be installed at {:?}",
        installed_bundle
    );

    // Step 6: Verify expected files from the bundle exist
    let bundle_manifest = installed_bundle.join("bundle.toml");
    assert!(
        bundle_manifest.exists(),
        "Installed bundle should contain bundle.toml"
    );

    let readme = installed_bundle.join("README.md");
    assert!(
        readme.exists(),
        "Installed bundle should contain README.md"
    );

    let assets_dir = installed_bundle.join("assets");
    assert!(
        assets_dir.exists(),
        "Installed bundle should contain assets directory"
    );

    // Step 7: Run gitf2 status command
    let status_output = run_gitf2(&["status"], &design_dir)?;
    assert!(
        status_output.status.success(),
        "gitf2 status should succeed"
    );

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    println!("Status output:\n{}", status_stdout);

    // Verify the bundle shows up in status
    assert!(
        status_stdout.contains("ui-assets") || status_stdout.contains("Synced"),
        "Status should show the installed bundle"
    );

    // Cleanup (commented out to inspect results in .tests folder)
    // cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

#[test]
#[ignore]
fn test_git_not_available_error_message() {
    // This test documents the expected behavior when git is not available
    // It's marked as ignored because we can't easily simulate git being unavailable
    
    // The check_preconditions function should return a clear error message
    // telling the user to install git or configure PATH
    
    // Manual test: rename git.exe temporarily and run the integration tests
    // to verify the error message is clear and helpful
}

#[test]
#[ignore]
fn test_install_with_specific_branch() -> Result<()> {
    check_preconditions()?;

    let test_name = "install_branch";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    create_sample_project(&test_dir)?;

    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    // Install from a specific branch using HTTPS
    bundles.insert(
        "ui-assets-main".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_BUNDLE_REPO_HTTPS.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
        },
    );

    let _manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Test branch-specific install"),
        None,
        bundles,
    )?;

    let output = run_gitf2(&["install"], &design_dir)?;

    assert!(
        output.status.success(),
        "gitf2 install with branch should succeed"
    );

    let installed_bundle = design_dir.join(BUNDLE_DIR).join("ui-assets-main");
    assert!(installed_bundle.exists(), "Bundle should be installed");

    // Cleanup (commented out to inspect results in .tests folder)
    // cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

#[test]
#[ignore]
fn test_status_shows_correct_state_after_install() -> Result<()> {
    check_preconditions()?;

    let test_name = "status_after_install";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    create_sample_project(&test_dir)?;

    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    bundles.insert(
        "ui-assets".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_BUNDLE_REPO_HTTPS.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
        },
    );

    create_bundle_manifest(&design_dir, None, None, bundles)?;

    // Install the bundle
    let install_output = run_gitf2(&["install"], &design_dir)?;
    assert!(install_output.status.success(), "Install should succeed");

    // Check status
    let status_output = run_gitf2(&["status"], &design_dir)?;
    assert!(status_output.status.success(), "Status should succeed");

    let stdout = String::from_utf8_lossy(&status_output.stdout);
    
    // After a fresh install, the bundle should be in "Synced" state
    assert!(
        stdout.contains("Synced") || stdout.contains("synced"),
        "Freshly installed bundle should be synced. Got: {}",
        stdout
    );

    // Now modify a file in the installed bundle to make it "unsynced"
    let installed_readme = design_dir
        .join(BUNDLE_DIR)
        .join("ui-assets")
        .join("README.md");
    
    if installed_readme.exists() {
        fs::write(&installed_readme, "# Modified content\n\nThis was changed locally.")?;
        
        // Check status again - should show unsynced or modified
        let status_after_modify = run_gitf2(&["status"], &design_dir)?;
        let stdout_after = String::from_utf8_lossy(&status_after_modify.stdout);
        
        println!("Status after modification:\n{}", stdout_after);
        // The status should indicate the bundle has local changes
    }

    // Cleanup (commented out to inspect results in .tests folder)
    // cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}
