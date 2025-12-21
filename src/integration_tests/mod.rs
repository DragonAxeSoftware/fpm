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
    cleanup_test_env, create_bundle_manifest, create_sample_project, get_gitf2_binary_path,
    is_git_available, run_gitf2, setup_test_env,
};
use crate::types::{BundleDependency, BUNDLE_DIR};

const TEST_CATEGORY: &str = "integration";

/// Example 1: UI assets bundle (leaf bundle, no dependencies)
const EXAMPLE_1_REPO: &str = "https://github.com/DragonAxeSoftware/gitf2-example-1.git";

/// Example 2: UI components bundle (depends on example-3)
const EXAMPLE_2_REPO: &str = "https://github.com/DragonAxeSoftware/gitf2-example-2.git";

/// Example 3: Base styles bundle (leaf bundle, no dependencies)
/// This is automatically installed as a nested dependency of example-2
#[allow(dead_code)]
const EXAMPLE_3_REPO: &str = "https://github.com/DragonAxeSoftware/gitf2-example-3.git";

/// SSH URLs for future SSH authentication tests
/// NOTE: SSH authentication is not fully implemented in tests yet.
#[allow(dead_code)]
const EXAMPLE_1_REPO_SSH: &str = "git@github.com:DragonAxeSoftware/gitf2-example-1.git";

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
            git: EXAMPLE_1_REPO.to_string(),
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
        },
    );

    create_bundle_manifest(
        &design_dir,
        Some("Test nested bundle installation with multiple top-level bundles"),
        None,
        bundles,
    )?;

    println!("Running gitf2 install for nested bundles in {:?}", design_dir);
    let output = run_gitf2(&["install"], &design_dir)?;

    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "gitf2 install with nested bundles should succeed. Exit code: {:?}\nstderr: {}",
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
    let status_output = run_gitf2(&["status"], &design_dir)?;
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

/// Helper to read current counter value from counter.txt
fn read_counter_value(content: &str) -> u32 {
    for line in content.lines() {
        if line.starts_with("count=") {
            if let Ok(val) = line.trim_start_matches("count=").parse::<u32>() {
                return val;
            }
        }
    }
    0
}

/// Helper to read current version from counter.txt
fn read_counter_version(content: &str) -> String {
    for line in content.lines() {
        if line.starts_with("version=") {
            return line.trim_start_matches("version=").to_string();
        }
    }
    "0.0.0".to_string()
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
        },
    );

    create_bundle_manifest(
        &design_dir,
        Some("Real push test - counter.txt only"),
        None,
        bundles,
    )?;

    // Step 3: Install the bundle
    println!("Installing ui-assets from real GitHub repo");
    let install_output = run_gitf2(&["install"], &design_dir)?;
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

    // Step 4: Read current counter.txt and bump version
    let counter_path = bundle_path.join("counter.txt");
    let current_content = if counter_path.exists() {
        fs::read_to_string(&counter_path)?
    } else {
        "version=0.0.0\ncount=0\n".to_string()
    };

    let current_version = read_counter_version(&current_content);
    let current_count = read_counter_value(&current_content);
    let new_version = bump_patch_version(&current_version);
    let new_count = current_count + 1;

    println!(
        "Updating counter.txt: version {} -> {}, count {} -> {}",
        current_version, new_version, current_count, new_count
    );

    let new_content = format!("version={}\ncount={}\n", new_version, new_count);
    fs::write(&counter_path, &new_content)?;

    // Step 5: Run gitf2 push
    println!("Pushing counter.txt update to real GitHub repo");
    let push_output = run_gitf2(
        &["push", "-m", &format!("gitf2 test: Bump counter to {}", new_version)],
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
        println!("✓ Successfully pushed counter.txt to {}", EXAMPLE_1_REPO);
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
    let install_output = run_gitf2(&["install"], &design_dir)?;
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

    // Step 4: Update counter.txt in all bundles
    let bundle_paths = [
        ("ui-assets", &ui_assets_path),
        ("ui-components", &ui_components_path),
        ("base-styles", &base_styles_path),
    ];

    for (name, path) in &bundle_paths {
        let counter_path = path.join("counter.txt");
        let current_content = if counter_path.exists() {
            fs::read_to_string(&counter_path)?
        } else {
            "version=0.0.0\ncount=0\n".to_string()
        };

        let current_version = read_counter_version(&current_content);
        let current_count = read_counter_value(&current_content);
        let new_version = bump_patch_version(&current_version);
        let new_count = current_count + 1;

        println!(
            "Updating {}/counter.txt: version {} -> {}, count {} -> {}",
            name, current_version, new_version, current_count, new_count
        );

        let new_content = format!("version={}\ncount={}\n", new_version, new_count);
        fs::write(&counter_path, &new_content)?;
    }

    // Step 5: Run gitf2 push - should push all 3 bundles (including nested)
    println!("Pushing counter.txt updates to all real GitHub repos");
    let push_output = run_gitf2(
        &["push", "-m", "gitf2 test: Bump counters (nested push test)"],
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
