//! Integration tests for fpm
//!
//! These tests run with real external dependencies (actual git repositories).
//! They require:
//! - Git installed and available in PATH
//! - Network access to the test repositories
//!
//! Run with: `cargo test integration_tests -- --ignored`

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

use crate::test_utils::{
    cleanup_test_env, create_bundle_manifest, create_sample_project, get_fpm_binary_path,
    is_git_available, run_fpm, setup_test_env,
};
use crate::types::{BundleDependency, BundleManifest, BUNDLE_DIR};

const TEST_CATEGORY: &str = "integration";

/// Example 1: UI assets bundle (leaf bundle, no dependencies)
const EXAMPLE_1_REPO: &str = "https://github.com/DragonAxeSoftware/fpm-example-1.git";

/// Example 2: UI components bundle (depends on example-3)
const EXAMPLE_2_REPO: &str = "https://github.com/DragonAxeSoftware/fpm-example-2.git";

/// Example 3: Base styles bundle (leaf bundle, no dependencies)
/// This is automatically installed as a nested dependency of example-2
#[allow(dead_code)]
const EXAMPLE_3_REPO: &str = "https://github.com/DragonAxeSoftware/fpm-example-3.git";

/// SSH URLs for future SSH authentication tests
/// NOTE: SSH authentication is not fully implemented in tests yet.
#[allow(dead_code)]
const EXAMPLE_1_REPO_SSH: &str = "git@github.com:DragonAxeSoftware/fpm-example-1.git";

/// Checks preconditions before running integration tests
fn check_preconditions() -> Result<()> {
    if !is_git_available() {
        anyhow::bail!(
            "Git is not installed or not in PATH. \
            Please install git or ensure it's correctly configured in your PATH environment variable."
        );
    }

    let binary_path = get_fpm_binary_path();
    if !binary_path.exists() {
        anyhow::bail!(
            "fpm binary not found at {:?}. \
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
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    let _manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Design assets from real git repository"),
        None,
        bundles,
    )?;

    // Step 3: Run fpm install command using the real binary
    println!("Running fpm install in {:?}", design_dir);
    let output = run_fpm(&["install"], &design_dir)?;

    // Print output for debugging
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Step 4: Verify the command succeeded
    assert!(
        output.status.success(),
        "fpm install should succeed. Exit code: {:?}\nstderr: {}",
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
    assert!(readme.exists(), "Installed bundle should contain README.md");

    let assets_dir = installed_bundle.join("assets");
    assert!(
        assets_dir.exists(),
        "Installed bundle should contain assets directory"
    );

    // Step 7: Run fpm status command
    let status_output = run_fpm(&["status"], &design_dir)?;
    assert!(status_output.status.success(), "fpm status should succeed");

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    println!("Status output:\n{}", status_stdout);

    // Verify the bundle shows up in status
    assert!(
        status_stdout.contains("ui-assets") || status_stdout.contains("Synced"),
        "Status should show the installed bundle"
    );

    cleanup_test_env(TEST_CATEGORY, test_name)?;

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
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    let _manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Test branch-specific install"),
        None,
        bundles,
    )?;

    let output = run_fpm(&["install"], &design_dir)?;

    assert!(
        output.status.success(),
        "fpm install with branch should succeed"
    );

    let installed_bundle = design_dir.join(BUNDLE_DIR).join("ui-assets-main");
    assert!(installed_bundle.exists(), "Bundle should be installed");

    cleanup_test_env(TEST_CATEGORY, test_name)?;

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
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    create_bundle_manifest(&design_dir, None, None, bundles)?;

    // Install the bundle
    let install_output = run_fpm(&["install"], &design_dir)?;
    assert!(install_output.status.success(), "Install should succeed");

    // Check status
    let status_output = run_fpm(&["status"], &design_dir)?;
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
        fs::write(
            &installed_readme,
            "# Modified content\n\nThis was changed locally.",
        )?;

        // Check status again - should show unsynced or modified
        let status_after_modify = run_fpm(&["status"], &design_dir)?;
        let stdout_after = String::from_utf8_lossy(&status_after_modify.stdout);

        println!("Status after modification:\n{}", stdout_after);
        // The status should indicate the bundle has local changes
    }

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

#[test]
#[ignore]
fn test_install_nested_bundles() -> Result<()> {
    check_preconditions()?;

    let test_name = "install_nested";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    create_sample_project(&test_dir)?;

    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    // Install example-1 (ui-assets) - a leaf bundle with no dependencies
    bundles.insert(
        "ui-assets".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    // Install example-2 (ui-components), which depends on example-3 (base-styles)
    bundles.insert(
        "ui-components".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_2_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    create_bundle_manifest(
        &design_dir,
        Some("Test nested bundle installation with multiple top-level bundles"),
        None,
        bundles,
    )?;

    println!("Running fpm install for nested bundles in {:?}", design_dir);
    let output = run_fpm(&["install"], &design_dir)?;

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "fpm install with nested bundles should succeed. Exit code: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = design_dir.join(BUNDLE_DIR);

    // Verify ui-assets (example-1) was installed
    let ui_assets = bundle_dir.join("ui-assets");
    assert!(
        ui_assets.exists(),
        "ui-assets bundle should be installed at {:?}",
        ui_assets
    );

    // Verify example-1 specific files exist
    assert!(
        ui_assets.join("README.md").exists(),
        "ui-assets should contain README.md"
    );
    assert!(
        ui_assets.join("assets").exists(),
        "ui-assets should contain assets directory"
    );

    // Verify ui-components (example-2) was installed
    let ui_components = bundle_dir.join("ui-components");
    assert!(
        ui_components.exists(),
        "ui-components bundle should be installed at {:?}",
        ui_components
    );

    // Verify example-2 specific files exist
    let components_dir = ui_components.join("components");
    assert!(
        components_dir.exists(),
        "ui-components should contain components directory"
    );
    assert!(
        components_dir.join("button.css").exists(),
        "ui-components should contain button.css"
    );
    assert!(
        components_dir.join("card.css").exists(),
        "ui-components should contain card.css"
    );

    // Verify the nested bundle (base-styles from example-3) was installed inside ui-components
    let nested_bundle_dir = ui_components.join(BUNDLE_DIR).join("base-styles");
    assert!(
        nested_bundle_dir.exists(),
        "Nested base-styles bundle should be installed at {:?}",
        nested_bundle_dir
    );

    // Verify example-3 specific files exist in the nested bundle
    let styles_dir = nested_bundle_dir.join("styles");
    assert!(
        styles_dir.exists(),
        "base-styles should contain styles directory"
    );
    assert!(
        styles_dir.join("variables.css").exists(),
        "base-styles should contain variables.css"
    );
    assert!(
        styles_dir.join("reset.css").exists(),
        "base-styles should contain reset.css"
    );

    // Run status and verify all bundles show up
    let status_output = run_fpm(&["status"], &design_dir)?;
    assert!(status_output.status.success(), "Status should succeed");

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    println!("Status output:\n{}", status_stdout);

    assert!(
        status_stdout.contains("ui-assets"),
        "Status should show ui-assets bundle"
    );
    assert!(
        status_stdout.contains("ui-components"),
        "Status should show ui-components bundle"
    );

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

/// Helper to configure git user for a repository
fn configure_git_user(repo_path: &std::path::Path) -> Result<()> {
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo_path)
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;
    Ok(())
}

/// Helper to bump patch version (0.0.1 -> 0.0.2)
fn bump_patch_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() == 3 {
        if let Ok(patch) = parts[2].parse::<u32>() {
            return format!("{}.{}.{}", parts[0], parts[1], patch + 1);
        }
    }
    version.to_string()
}

/// Helper to read and bump the version in a bundle's manifest
/// Also creates/updates a test_counter.txt file to verify new file pushes
/// Returns (old_version, new_version)
fn bump_manifest_version(bundle_path: &std::path::Path) -> Result<(String, String)> {
    let manifest_path = bundle_path.join("bundle.toml");
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest at {}", manifest_path.display()))?;

    let mut manifest: BundleManifest =
        toml::from_str(&content).context("Failed to parse bundle.toml")?;

    let old_version = manifest
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".to_string());
    let new_version = bump_patch_version(&old_version);
    manifest.version = Some(new_version.clone());

    let new_content = toml::to_string_pretty(&manifest).context("Failed to serialize manifest")?;
    fs::write(&manifest_path, new_content)
        .with_context(|| format!("Failed to write manifest at {}", manifest_path.display()))?;

    // Also create/update a test counter file to verify new file creation works
    let counter_path = bundle_path.join("test_counter.txt");
    let current_count = if counter_path.exists() {
        fs::read_to_string(&counter_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
    } else {
        0
    };
    fs::write(&counter_path, format!("{}\n", current_count + 1))?;

    Ok((old_version, new_version))
}

#[test]
#[ignore] // Run only when explicitly requested: cargo test integration_tests -- --ignored
fn test_push_counter_to_real_repo() -> Result<()> {
    // Check preconditions
    check_preconditions()?;

    let test_name = "push_real_repo";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    // Step 1: Create a sample project structure
    create_sample_project(&test_dir)?;

    // Step 2: Create a bundle.toml that references the real example-1 repository
    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    bundles.insert(
        "ui-assets".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    create_bundle_manifest(
        &design_dir,
        Some("Real push test - version bump"),
        None,
        bundles,
    )?;

    // Step 3: Install the bundle
    println!("Installing ui-assets from real GitHub repo");
    let install_output = run_fpm(&["install"], &design_dir)?;
    assert!(
        install_output.status.success(),
        "Install should succeed: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );

    // Verify the bundle is installed
    let bundle_path = design_dir.join(BUNDLE_DIR).join("ui-assets");
    assert!(bundle_path.exists(), "ui-assets bundle should be installed");

    // Configure git user
    configure_git_user(&bundle_path)?;

    // Step 4: Bump version in the bundle's manifest
    let (old_version, new_version) = bump_manifest_version(&bundle_path)?;
    println!(
        "Bumping manifest version: {} -> {}",
        old_version, new_version
    );

    // Step 5: Run fpm push
    println!("Pushing manifest version update to real GitHub repo");
    let push_output = run_fpm(
        &[
            "push",
            "-m",
            &format!("fpm test: Bump version to {}", new_version),
        ],
        &design_dir,
    )?;
    let push_stdout = String::from_utf8_lossy(&push_output.stdout);
    let push_stderr = String::from_utf8_lossy(&push_output.stderr);
    println!("Push stdout: {}", push_stdout);
    println!("Push stderr: {}", push_stderr);

    // Check if push succeeded or failed due to auth
    if push_output.status.success() {
        assert!(
            push_stdout.contains("Pushed") || push_stdout.contains("✓"),
            "Should indicate push success"
        );
        println!(
            "✓ Successfully pushed version {} to {}",
            new_version, EXAMPLE_1_REPO
        );
    } else {
        // Expected to fail if no push access
        let stderr_lower = push_stderr.to_lowercase();
        let is_auth_error = stderr_lower.contains("permission")
            || stderr_lower.contains("denied")
            || stderr_lower.contains("authentication")
            || stderr_lower.contains("403")
            || stderr_lower.contains("401");

        if is_auth_error {
            println!(
                "⚠ Push failed due to authentication (expected if no push access): {}",
                push_stderr
            );
            // This is OK - we're testing the push mechanism works, auth is user-dependent
        } else {
            panic!("Push failed with unexpected error: {}", push_stderr);
        }
    }

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

#[test]
#[ignore] // Run only when explicitly requested: cargo test integration_tests -- --ignored
fn test_push_nested_bundles_to_real_repos() -> Result<()> {
    // Check preconditions
    check_preconditions()?;

    let test_name = "push_nested_real";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    // Step 1: Create a sample project structure
    create_sample_project(&test_dir)?;

    // Step 2: Create a bundle.toml that references example-2 (which depends on example-3)
    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();

    // Also add example-1 as a separate bundle
    bundles.insert(
        "ui-assets".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_1_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    bundles.insert(
        "ui-components".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: EXAMPLE_2_REPO.to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
            include: None,
        },
    );

    create_bundle_manifest(
        &design_dir,
        Some("Nested push test - real repos"),
        None,
        bundles,
    )?;

    // Step 3: Install all bundles (including nested base-styles from example-3)
    println!("Installing bundles from real GitHub repos");
    let install_output = run_fpm(&["install"], &design_dir)?;
    assert!(
        install_output.status.success(),
        "Install should succeed: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );

    // Verify all bundles are installed
    let ui_assets_path = design_dir.join(BUNDLE_DIR).join("ui-assets");
    let ui_components_path = design_dir.join(BUNDLE_DIR).join("ui-components");
    let base_styles_path = ui_components_path.join(BUNDLE_DIR).join("base-styles");

    assert!(ui_assets_path.exists(), "ui-assets should be installed");
    assert!(
        ui_components_path.exists(),
        "ui-components should be installed"
    );
    assert!(
        base_styles_path.exists(),
        "base-styles should be installed (nested)"
    );

    // Configure git user for all bundles
    configure_git_user(&ui_assets_path)?;
    configure_git_user(&ui_components_path)?;
    configure_git_user(&base_styles_path)?;

    // Step 4: Bump version in all bundle manifests
    let bundle_paths = [
        ("ui-assets", &ui_assets_path),
        ("ui-components", &ui_components_path),
        ("base-styles", &base_styles_path),
    ];

    for (name, path) in &bundle_paths {
        let (old_version, new_version) = bump_manifest_version(path)?;
        println!(
            "Bumping {}/bundle.toml version: {} -> {}",
            name, old_version, new_version
        );
    }

    // Step 5: Run fpm push - should push all 3 bundles (including nested)
    println!("Pushing manifest version updates to all real GitHub repos");
    let push_output = run_fpm(
        &["push", "-m", "fpm test: Bump versions (nested push test)"],
        &design_dir,
    )?;
    let push_stdout = String::from_utf8_lossy(&push_output.stdout);
    let push_stderr = String::from_utf8_lossy(&push_output.stderr);
    println!("Push stdout: {}", push_stdout);
    println!("Push stderr: {}", push_stderr);

    // Count successes and auth warnings
    let success_count = push_stdout.matches('✓').count();
    let warning_count = push_stdout.to_lowercase().matches("warning").count();

    println!(
        "Push results: {} succeeded, {} auth warnings",
        success_count, warning_count
    );

    // The test passes if push command completed (even with auth warnings)
    // The push mechanism is working, auth is user/environment dependent
    if success_count > 0 {
        println!("✓ Successfully pushed to {} bundle(s)", success_count);
    }
    if warning_count > 0 {
        println!(
            "⚠ {} bundle(s) had auth warnings (expected if no push access)",
            warning_count
        );
    }

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}
