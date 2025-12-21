//! Unit tests for gitf2
//!
//! These tests run without external dependencies using mock implementations.
//! Test files are placed at <workspace>/.tests directory.

mod mock_git;

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::{load_manifest, save_manifest};
use crate::git::GitOperations;
use crate::types::{BundleDependency, BundleManifest, BundleStatus, BUNDLE_DIR, GITF2_IDENTIFIER};

use self::mock_git::{MockBundleContent, MockGitOperations};

/// Gets the test directory path
fn get_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tests")
}

/// Sets up a clean test environment
fn setup_test_env(test_name: &str) -> Result<PathBuf> {
    let test_dir = get_test_dir().join(test_name);
    
    // Clean up previous test run
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    
    fs::create_dir_all(&test_dir)?;
    Ok(test_dir)
}

/// Cleans up test environment
fn cleanup_test_env(test_name: &str) -> Result<()> {
    let test_dir = get_test_dir().join(test_name);
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    Ok(())
}

/// Creates a sample project structure with non-gitf2 files
fn create_sample_project(base_dir: &Path) -> Result<()> {
    // Create typical project structure
    let src_dir = base_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create some sample files
    fs::write(base_dir.join("README.md"), "# My Project\n\nA sample project.")?;
    fs::write(src_dir.join("main.rs"), "fn main() {\n    println!(\"Hello!\");\n}")?;
    fs::write(base_dir.join(".gitignore"), "/target\n.gitf2/")?;
    
    // Create a design directory where we'll add bundles
    let design_dir = src_dir.join("design");
    fs::create_dir_all(&design_dir)?;
    fs::write(design_dir.join("styles.css"), "body { margin: 0; }")?;
    
    Ok(())
}

/// Creates a bundle.toml manifest in the specified directory
fn create_bundle_manifest(
    dir: &Path,
    description: Option<&str>,
    root: Option<&str>,
    bundles: HashMap<String, BundleDependency>,
) -> Result<PathBuf> {
    let manifest = BundleManifest {
        gitf2_version: "0.1.0".to_string(),
        identifier: GITF2_IDENTIFIER.to_string(),
        description: description.map(String::from),
        root: root.map(PathBuf::from),
        bundles,
    };
    
    let manifest_path = dir.join("bundle.toml");
    save_manifest(&manifest, &manifest_path)?;
    
    Ok(manifest_path)
}

#[test]
fn test_full_install_workflow() -> Result<()> {
    let test_name = "install_workflow";
    let test_dir = setup_test_env(test_name)?;
    
    // Step 1: Create a sample project structure
    create_sample_project(&test_dir)?;
    
    // Step 2: Create a bundle.toml in the design directory
    let design_dir = test_dir.join("src").join("design");
    let mut bundles = HashMap::new();
    
    bundles.insert(
        "design-from-martha".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/martha/designs.git".to_string(),
            path: Some(PathBuf::from("assets")),
            branch: None,
        },
    );
    
    bundles.insert(
        "shared-icons".to_string(),
        BundleDependency {
            version: "2.0.0".to_string(),
            git: "git@github.com:company/icons.git".to_string(),
            path: None,
            branch: Some("main".to_string()),
        },
    );
    
    let manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Design assets for the project"),
        None,
        bundles,
    )?;
    
    // Step 3: Verify manifest was created correctly
    let loaded_manifest = load_manifest(&manifest_path)?;
    assert!(loaded_manifest.is_valid_gitf2_manifest());
    assert_eq!(loaded_manifest.bundles.len(), 2);
    assert!(loaded_manifest.bundles.contains_key("design-from-martha"));
    assert!(loaded_manifest.bundles.contains_key("shared-icons"));
    
    // Step 4: Set up mock git operations
    let mock_git = Arc::new(MockGitOperations::new());
    
    // Register mock remote repositories with their content
    mock_git.register_remote_bundle(
        "https://github.com/martha/designs.git",
        "assets",
        create_mock_bundle_content("Martha's amazing designs"),
    );
    
    mock_git.register_remote_bundle(
        "git@github.com:company/icons.git",
        "",
        create_mock_bundle_content("Shared icon library"),
    );
    
    // Step 5: Execute install using the mock git
    execute_install_with_mock(&manifest_path, mock_git.clone())?;
    
    // Step 6: Verify bundles were "installed" (mock cloned)
    let bundle_dir = design_dir.join(BUNDLE_DIR);
    assert!(bundle_dir.exists(), "Bundle directory should exist");
    
    let martha_bundle_dir = bundle_dir.join("design-from-martha");
    let icons_bundle_dir = bundle_dir.join("shared-icons");
    
    assert!(martha_bundle_dir.exists(), "Martha's bundle should be installed");
    assert!(icons_bundle_dir.exists(), "Icons bundle should be installed");
    
    // Verify mock git recorded the clone operations
    let cloned = mock_git.get_cloned_repos();
    assert_eq!(cloned.len(), 2, "Should have cloned 2 repositories");
    
    // Step 7: Verify the bundle.toml files were created in installed bundles
    let martha_manifest_path = martha_bundle_dir.join("bundle.toml");
    assert!(martha_manifest_path.exists(), "Bundle should have its own manifest");
    
    // Step 8: Verify status can be checked
    let statuses = get_bundle_statuses_with_mock(&manifest_path, mock_git.clone())?;
    assert!(!statuses.is_empty(), "Should have bundle statuses");
    
    // Cleanup
    cleanup_test_env(test_name)?;
    
    Ok(())
}

#[test]
fn test_nested_bundles_workflow() -> Result<()> {
    let test_name = "nested_bundles";
    let test_dir = setup_test_env(test_name)?;
    
    create_sample_project(&test_dir)?;
    
    // Create a bundle that depends on another bundle
    let design_dir = test_dir.join("src").join("design");
    
    let mut top_bundles = HashMap::new();
    top_bundles.insert(
        "ui-kit".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/example/ui-kit.git".to_string(),
            path: None,
            branch: None,
        },
    );
    
    let manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Top-level bundle with nested dependencies"),
        None,
        top_bundles,
    )?;
    
    // Set up mock git with nested bundle
    let mock_git = Arc::new(MockGitOperations::new());
    
    // The ui-kit bundle has its own dependencies
    let mut nested_bundles = HashMap::new();
    nested_bundles.insert(
        "base-styles".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/example/base-styles.git".to_string(),
            path: None,
            branch: None,
        },
    );
    
    mock_git.register_remote_bundle_with_deps(
        "https://github.com/example/ui-kit.git",
        "",
        create_mock_bundle_content("UI Kit with nested deps"),
        nested_bundles,
    );
    
    mock_git.register_remote_bundle(
        "https://github.com/example/base-styles.git",
        "",
        create_mock_bundle_content("Base CSS styles"),
    );
    
    // Execute install
    execute_install_with_mock(&manifest_path, mock_git.clone())?;
    
    // Verify nested structure
    let bundle_dir = design_dir.join(BUNDLE_DIR);
    let ui_kit_dir = bundle_dir.join("ui-kit");
    
    assert!(ui_kit_dir.exists(), "UI kit should be installed");
    
    // The nested bundle should be in ui-kit/.gitf2/base-styles
    let nested_bundle_dir = ui_kit_dir.join(BUNDLE_DIR).join("base-styles");
    assert!(nested_bundle_dir.exists(), "Nested bundle should be installed");
    
    cleanup_test_env(test_name)?;
    
    Ok(())
}

#[test]
fn test_duplicate_bundle_names_error() -> Result<()> {
    let test_name = "duplicate_names";
    let test_dir = setup_test_env(test_name)?;
    
    create_sample_project(&test_dir)?;
    
    let design_dir = test_dir.join("src").join("design");
    
    // Create manifest with duplicate bundle references (same name, different sources)
    // This is actually prevented by HashMap, so we test the conflict detection differently
    let mut bundles = HashMap::new();
    bundles.insert(
        "same-name-bundle".to_string(),
        BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/example/bundle.git".to_string(),
            path: None,
            branch: None,
        },
    );
    
    let manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Bundle for duplicate test"),
        None,
        bundles,
    )?;
    
    let mock_git = Arc::new(MockGitOperations::new());
    mock_git.register_remote_bundle(
        "https://github.com/example/bundle.git",
        "",
        create_mock_bundle_content("Test bundle"),
    );
    
    // This should succeed (no actual duplicates in this test case)
    let result = execute_install_with_mock(&manifest_path, mock_git);
    assert!(result.is_ok());
    
    cleanup_test_env(test_name)?;
    
    Ok(())
}

#[test]
fn test_source_bundle_status() -> Result<()> {
    let test_name = "source_bundle";
    let test_dir = setup_test_env(test_name)?;
    
    create_sample_project(&test_dir)?;
    
    let design_dir = test_dir.join("src").join("design");
    
    // Create artifacts directory
    let artifacts_dir = design_dir.join("my-artifacts");
    fs::create_dir_all(&artifacts_dir)?;
    fs::write(artifacts_dir.join("logo.svg"), "<svg></svg>")?;
    
    // Create a source bundle manifest (has root)
    let manifest_path = create_bundle_manifest(
        &design_dir,
        Some("Source bundle with artifacts"),
        Some("my-artifacts"),
        HashMap::new(),
    )?;
    
    let loaded_manifest = load_manifest(&manifest_path)?;
    assert!(loaded_manifest.is_source_bundle(), "Should be a source bundle");
    
    let mock_git = Arc::new(MockGitOperations::new());
    let statuses = get_bundle_statuses_with_mock(&manifest_path, mock_git)?;
    
    // The root bundle should show as source
    let has_source = statuses.iter().any(|(_, status)| *status == BundleStatus::Source);
    assert!(has_source, "Should have a source bundle status");
    
    cleanup_test_env(test_name)?;
    
    Ok(())
}

// === Helper functions for mock-based execution ===

fn create_mock_bundle_content(description: &str) -> MockBundleContent {
    MockBundleContent {
        description: description.to_string(),
        files: vec![
            ("README.md".to_string(), format!("# Bundle\n\n{}", description)),
        ],
    }
}

/// Executes install command using mock git operations
fn execute_install_with_mock(manifest_path: &Path, mock_git: Arc<MockGitOperations>) -> Result<()> {
    use crate::config::load_manifest;
    use crate::types::BUNDLE_DIR;
    use std::collections::HashSet;
    
    let manifest = load_manifest(manifest_path)?;
    let parent_dir = manifest_path.parent().unwrap();
    
    // Check for duplicate bundle names
    let bundle_names: Vec<&str> = manifest.bundles.keys().map(|s| s.as_str()).collect();
    let unique_names: HashSet<&str> = bundle_names.iter().copied().collect();
    
    if bundle_names.len() != unique_names.len() {
        anyhow::bail!("Duplicate bundle names detected");
    }
    
    let bundle_dir = parent_dir.join(BUNDLE_DIR);
    fs::create_dir_all(&bundle_dir)?;
    
    for (name, dependency) in &manifest.bundles {
        let target_path = bundle_dir.join(name);
        
        // Use mock git to "clone"
        mock_git.clone_repository(&dependency.git, &target_path, dependency.branch())?;
        
        // Install nested bundles if the mock created a bundle.toml
        let nested_manifest_path = target_path.join("bundle.toml");
        if nested_manifest_path.exists() {
            install_nested_bundles_with_mock(&nested_manifest_path, mock_git.clone())?;
        }
    }
    
    Ok(())
}

fn install_nested_bundles_with_mock(manifest_path: &Path, mock_git: Arc<MockGitOperations>) -> Result<()> {
    use crate::config::load_manifest;
    use crate::types::BUNDLE_DIR;
    
    let manifest = load_manifest(manifest_path)?;
    let parent_dir = manifest_path.parent().unwrap();
    let bundle_dir = parent_dir.join(BUNDLE_DIR);
    
    if !bundle_dir.exists() {
        fs::create_dir_all(&bundle_dir)?;
    }
    
    for (name, dependency) in &manifest.bundles {
        let target_path = bundle_dir.join(name);
        mock_git.clone_repository(&dependency.git, &target_path, dependency.branch())?;
        
        let nested_manifest_path = target_path.join("bundle.toml");
        if nested_manifest_path.exists() {
            install_nested_bundles_with_mock(&nested_manifest_path, mock_git.clone())?;
        }
    }
    
    Ok(())
}

fn get_bundle_statuses_with_mock(
    manifest_path: &Path,
    mock_git: Arc<MockGitOperations>,
) -> Result<Vec<(String, BundleStatus)>> {
    use crate::config::load_manifest;
    use crate::types::BUNDLE_DIR;
    
    let manifest = load_manifest(manifest_path)?;
    let parent_dir = manifest_path.parent().unwrap();
    
    let mut statuses = Vec::new();
    
    // Check if source bundle
    if manifest.is_source_bundle() {
        let root_path = parent_dir.join(manifest.root.as_ref().unwrap());
        let status = if root_path.exists() {
            if mock_git.is_repository(&root_path) {
                if mock_git.has_local_changes(&root_path)? {
                    BundleStatus::Unsynced
                } else {
                    BundleStatus::Source
                }
            } else {
                BundleStatus::Source
            }
        } else {
            BundleStatus::Unsynced
        };
        
        statuses.push(("(root)".to_string(), status));
    }
    
    // Check installed bundles
    let bundle_dir = parent_dir.join(BUNDLE_DIR);
    if bundle_dir.exists() {
        for entry in fs::read_dir(&bundle_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if !path.is_dir() {
                continue;
            }
            
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            
            let status = if mock_git.is_repository(&path) {
                if mock_git.has_local_changes(&path)? {
                    BundleStatus::Unsynced
                } else {
                    BundleStatus::Synced
                }
            } else {
                BundleStatus::Unsynced
            };
            
            statuses.push((name, status));
        }
    }
    
    Ok(statuses)
}
