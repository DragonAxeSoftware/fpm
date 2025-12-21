//! Local integration tests for fpm
//!
//! These tests run with local git repositories (no network required).
//! They test the same functionality as integration tests but use local
//! bare git repos as "remotes" instead of real GitHub repositories.
//!
//! This allows testing push/pull workflows without:
//! - Network access
//! - Push credentials to real repos
//! - Risk of modifying real repositories
//!
//! Run with: `cargo test local_integration_tests --lib`
//! Or use the script: `.\scripts\tests\run_local_integration_tests.ps1`

use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use crate::test_utils::{
    cleanup_test_env, create_bundle_manifest, create_sample_project, get_fpm_binary_path,
    is_git_available, run_fpm, setup_test_env,
};
use crate::types::{BundleDependency, BUNDLE_DIR};

const TEST_CATEGORY: &str = "local_integration";

/// Checks preconditions before running local integration tests
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
            Please run 'cargo build' first.",
            binary_path
        );
    }

    Ok(())
}

/// Helper to create a local bare git repo with initial content
fn setup_local_bare_repo(
    remote_dir: &std::path::Path,
    setup_clone_dir: &std::path::Path,
    manifest_content: &str,
    counter_content: &str,
) -> Result<()> {
    fs::create_dir_all(remote_dir)?;

    // Initialize bare repo with main as default branch
    let init_output = std::process::Command::new("git")
        .args(["init", "--bare", "--initial-branch=main"])
        .current_dir(remote_dir)
        .output()?;
    assert!(
        init_output.status.success(),
        "Failed to init bare repo: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    // Clone the bare repo
    let clone_output = std::process::Command::new("git")
        .args([
            "clone",
            remote_dir.to_str().unwrap(),
            setup_clone_dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(
        clone_output.status.success(),
        "Failed to clone bare repo: {}",
        String::from_utf8_lossy(&clone_output.stderr)
    );

    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(setup_clone_dir)
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(setup_clone_dir)
        .output()?;

    // Create main branch (needed for empty repo)
    std::process::Command::new("git")
        .args(["checkout", "-b", "main"])
        .current_dir(setup_clone_dir)
        .output()?;

    // Write initial content
    fs::write(setup_clone_dir.join("bundle.toml"), manifest_content)?;
    fs::write(setup_clone_dir.join("counter.txt"), counter_content)?;
    fs::write(setup_clone_dir.join("README.md"), "# Test Bundle\n")?;

    // Commit and push
    let add_output = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(setup_clone_dir)
        .output()?;
    assert!(
        add_output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&add_output.stderr)
    );

    let commit_output = std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(setup_clone_dir)
        .output()?;
    assert!(
        commit_output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&commit_output.stderr)
    );

    let push_output = std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(setup_clone_dir)
        .output()?;
    assert!(
        push_output.status.success(),
        "git push failed: {}",
        String::from_utf8_lossy(&push_output.stderr)
    );

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

#[test]
fn test_push_bundle_changes_local() -> Result<()> {
    // Check preconditions
    check_preconditions()?;

    let test_name = "push_bundle_local";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    // Step 1: Create a bare git repository to act as our "remote"
    let remote_dir = test_dir.join("remote");
    let setup_clone = test_dir.join("setup_clone");

    let bundle_manifest = r#"fpm_version = "0.1.0"
identifier = "fpm-bundle"
description = "Test bundle for push command"

[bundles]
"#;
    setup_local_bare_repo(&remote_dir, &setup_clone, bundle_manifest, "version=0.0.1\ncount=0\n")?;

    // Step 2: Create a sample project that uses this bundle
    create_sample_project(&test_dir.join("project"))?;
    let project_dir = test_dir.join("project");
    let design_dir = project_dir.join("src").join("design");
    fs::create_dir_all(&design_dir)?;

    let mut bundles = HashMap::new();
    bundles.insert(
        "push-test".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: remote_dir.to_str().unwrap().to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
        },
    );

    create_bundle_manifest(&design_dir, Some("Push test bundle"), None, bundles)?;

    // Step 3: Install the bundle
    println!("Installing bundle from local 'remote'");
    let install_output = run_fpm(&["install"], &design_dir)?;
    println!(
        "Install stdout: {}",
        String::from_utf8_lossy(&install_output.stdout)
    );
    println!(
        "Install stderr: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );
    assert!(
        install_output.status.success(),
        "Install should succeed. stderr: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );

    // Verify the bundle is installed
    let bundle_path = design_dir.join(BUNDLE_DIR).join("push-test");
    assert!(bundle_path.exists(), "push-test bundle should be installed");
    assert!(
        bundle_path.join("counter.txt").exists(),
        "counter.txt should exist"
    );

    // Configure git user for the installed bundle
    configure_git_user(&bundle_path)?;

    // Step 4: Make local changes to counter.txt (simulate user edits)
    let updated_counter = "version=0.0.2\ncount=1\n";
    fs::write(bundle_path.join("counter.txt"), updated_counter)?;

    // Step 5: Run fpm push
    println!("Pushing bundle changes");
    let push_output = run_fpm(&["push", "-m", "Bump counter to 0.0.2"], &design_dir)?;
    let push_stdout = String::from_utf8_lossy(&push_output.stdout);
    let push_stderr = String::from_utf8_lossy(&push_output.stderr);
    println!("Push stdout: {}", push_stdout);
    println!("Push stderr: {}", push_stderr);

    assert!(push_output.status.success(), "Push should succeed");
    assert!(
        push_stdout.contains("Pushed") || push_stdout.contains("✓"),
        "Should indicate push success"
    );

    // Step 6: Verify the change was pushed to the remote
    let verify_clone = test_dir.join("verify_clone");
    let verify_output = std::process::Command::new("git")
        .args([
            "clone",
            remote_dir.to_str().unwrap(),
            verify_clone.to_str().unwrap(),
        ])
        .output()?;
    assert!(
        verify_output.status.success(),
        "Failed to clone for verification"
    );

    let remote_counter = fs::read_to_string(verify_clone.join("counter.txt"))?;
    assert!(
        remote_counter.contains("version=0.0.2"),
        "Remote should have updated version. Got: {}",
        remote_counter
    );
    assert!(
        remote_counter.contains("count=1"),
        "Remote should have updated count. Got: {}",
        remote_counter
    );

    // Check git log for our commit message
    let log_output = std::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&verify_clone)
        .output()?;
    let log_msg = String::from_utf8_lossy(&log_output.stdout);
    assert!(
        log_msg.contains("Bump counter to 0.0.2"),
        "Commit message should be in remote log. Got: {}",
        log_msg
    );

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}

#[test]
fn test_push_nested_bundles_local() -> Result<()> {
    // Check preconditions
    check_preconditions()?;

    let test_name = "push_nested_local";
    let test_dir = setup_test_env(TEST_CATEGORY, test_name)?;

    // Step 1: Create two bare git repositories (parent and child)
    let parent_remote = test_dir.join("parent_remote");
    let child_remote = test_dir.join("child_remote");

    // Setup child bundle
    let child_manifest = r#"fpm_version = "0.1.0"
identifier = "fpm-bundle"
description = "Child bundle"

[bundles]
"#;
    setup_local_bare_repo(
        &child_remote,
        &test_dir.join("child_setup"),
        child_manifest,
        "child_version=0.0.1\n",
    )?;

    // Setup parent bundle (depends on child)
    let parent_manifest = format!(
        r#"fpm_version = "0.1.0"
identifier = "fpm-bundle"
description = "Parent bundle with child dependency"

[bundles.child-bundle]
version = "1.0.0"
git = "{}"
branch = "main"
"#,
        child_remote.to_str().unwrap().replace('\\', "/")
    );
    setup_local_bare_repo(
        &parent_remote,
        &test_dir.join("parent_setup"),
        &parent_manifest,
        "parent_version=0.0.1\n",
    )?;

    // Step 2: Create project and install the parent bundle
    create_sample_project(&test_dir.join("project"))?;
    let project_dir = test_dir.join("project");
    let design_dir = project_dir.join("src").join("design");
    fs::create_dir_all(&design_dir)?;

    let mut bundles = HashMap::new();
    bundles.insert(
        "parent-bundle".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: parent_remote.to_str().unwrap().to_string(),
            path: None,
            branch: Some("main".to_string()),
            ssh_key: None,
        },
    );
    create_bundle_manifest(&design_dir, Some("Nested push test"), None, bundles)?;

    // Install bundles
    let install_output = run_fpm(&["install"], &design_dir)?;
    assert!(
        install_output.status.success(),
        "Install should succeed: {}",
        String::from_utf8_lossy(&install_output.stderr)
    );

    // Verify both bundles installed
    let parent_path = design_dir.join(BUNDLE_DIR).join("parent-bundle");
    let child_path = parent_path.join(BUNDLE_DIR).join("child-bundle");
    assert!(parent_path.exists(), "Parent bundle should be installed");
    assert!(
        child_path.exists(),
        "Child bundle should be installed (nested)"
    );

    // Configure git for both bundles
    configure_git_user(&parent_path)?;
    configure_git_user(&child_path)?;

    // Step 3: Modify counter.txt in both bundles
    fs::write(parent_path.join("counter.txt"), "parent_version=0.0.2\n")?;
    fs::write(child_path.join("counter.txt"), "child_version=0.0.2\n")?;

    // Step 4: Run push - should push both nested bundles
    let push_output = run_fpm(&["push", "-m", "Bump to 0.0.2"], &design_dir)?;
    let push_stdout = String::from_utf8_lossy(&push_output.stdout);
    let push_stderr = String::from_utf8_lossy(&push_output.stderr);
    println!("Push stdout: {}", push_stdout);
    println!("Push stderr: {}", push_stderr);

    assert!(push_output.status.success(), "Push should succeed");
    assert!(
        push_stdout.contains("2 bundle(s)")
            || (push_stdout.contains("✓") && push_stdout.matches('✓').count() >= 2),
        "Should push 2 bundles. Got: {}",
        push_stdout
    );

    // Step 5: Verify changes in both remotes
    let verify_parent = test_dir.join("verify_parent");
    std::process::Command::new("git")
        .args([
            "clone",
            parent_remote.to_str().unwrap(),
            verify_parent.to_str().unwrap(),
        ])
        .output()?;
    let parent_counter = fs::read_to_string(verify_parent.join("counter.txt"))?;
    assert!(
        parent_counter.contains("parent_version=0.0.2"),
        "Parent remote should have updated version. Got: {}",
        parent_counter
    );

    let verify_child = test_dir.join("verify_child");
    std::process::Command::new("git")
        .args([
            "clone",
            child_remote.to_str().unwrap(),
            verify_child.to_str().unwrap(),
        ])
        .output()?;
    let child_counter = fs::read_to_string(verify_child.join("counter.txt"))?;
    assert!(
        child_counter.contains("child_version=0.0.2"),
        "Child remote should have updated version. Got: {}",
        child_counter
    );

    cleanup_test_env(TEST_CATEGORY, test_name)?;

    Ok(())
}
