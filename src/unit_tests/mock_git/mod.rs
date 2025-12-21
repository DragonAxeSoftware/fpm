//! Mock implementation of Git operations for testing
//! 
//! This module provides a mock git implementation that simulates git operations
//! without actually connecting to remote repositories.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::config::save_manifest;
use crate::git::GitOperations;
use crate::types::{BundleDependency, BundleManifest, GITF2_IDENTIFIER};

/// Content for a mock bundle
pub struct MockBundleContent {
    pub description: String,
    pub files: Vec<(String, String)>,
}

/// Registration for a remote bundle
struct RemoteBundleRegistration {
    #[allow(dead_code)]
    url: String,
    #[allow(dead_code)]
    path: String,
    content: MockBundleContent,
    nested_bundles: HashMap<String, BundleDependency>,
}

/// Mock git operations for testing
/// 
/// This implementation doesn't actually perform git operations but instead:
/// - Tracks which repositories were "cloned"
/// - Creates local directory structures to simulate cloned repos
/// - Supports nested bundle dependencies
pub struct MockGitOperations {
    /// Registered remote bundles (url -> registration)
    _remotes: RwLock<HashMap<String, RemoteBundleRegistration>>,
    
    /// Paths that have been "cloned"
    _cloned_repos: RwLock<Vec<ClonedRepo>>,
    
    /// Paths that have been initialized as repos
    _initialized_repos: RwLock<Vec<PathBuf>>,
    
    /// Simulated local changes (path -> has changes)
    _local_changes: RwLock<HashMap<PathBuf, bool>>,
}

#[derive(Clone)]
#[allow(dead_code)] // Fields kept for potential test assertions
pub struct ClonedRepo {
    pub url: String,
    pub path: PathBuf,
    pub branch: String,
}

impl MockGitOperations {
    pub fn new() -> Self {
        Self {
            _remotes: RwLock::new(HashMap::new()),
            _cloned_repos: RwLock::new(Vec::new()),
            _initialized_repos: RwLock::new(Vec::new()),
            _local_changes: RwLock::new(HashMap::new()),
        }
    }
    
    /// Registers a remote bundle that can be "cloned"
    pub fn register_remote_bundle(&self, url: &str, path: &str, content: MockBundleContent) {
        self.register_remote_bundle_with_deps(url, path, content, HashMap::new());
    }
    
    /// Registers a remote bundle with nested dependencies
    pub fn register_remote_bundle_with_deps(
        &self,
        url: &str,
        path: &str,
        content: MockBundleContent,
        nested_bundles: HashMap<String, BundleDependency>,
    ) {
        let mut remotes = self._remotes.write().unwrap();
        remotes.insert(
            url.to_string(),
            RemoteBundleRegistration {
                url: url.to_string(),
                path: path.to_string(),
                content,
                nested_bundles,
            },
        );
    }
    
    /// Returns the list of cloned repositories
    pub fn get_cloned_repos(&self) -> Vec<ClonedRepo> {
        self._cloned_repos.read().unwrap().clone()
    }
    
    /// Simulates local changes for a path
    #[allow(dead_code)]
    pub fn set_local_changes(&self, path: &Path, has_changes: bool) {
        let mut changes = self._local_changes.write().unwrap();
        changes.insert(path.to_path_buf(), has_changes);
    }
    
    /// Creates mock bundle files at the target path
    fn create_mock_bundle_files(
        &self,
        target_path: &Path,
        registration: &RemoteBundleRegistration,
    ) -> Result<()> {
        fs::create_dir_all(target_path)?;
        
        // Write content files
        for (filename, content) in &registration.content.files {
            let file_path = target_path.join(filename);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, content)?;
        }
        
        // Create bundle.toml manifest
        let manifest = BundleManifest {
            gitf2_version: "0.1.0".to_string(),
            identifier: GITF2_IDENTIFIER.to_string(),
            description: Some(registration.content.description.clone()),
            root: None,
            bundles: registration.nested_bundles.clone(),
        };
        
        let manifest_path = target_path.join("bundle.toml");
        save_manifest(&manifest, &manifest_path)?;
        
        // Mark as initialized repo
        let mut initialized = self._initialized_repos.write().unwrap();
        initialized.push(target_path.to_path_buf());
        
        Ok(())
    }
}

impl GitOperations for MockGitOperations {
    fn clone_repository(&self, url: &str, path: &Path, branch: &str) -> Result<()> {
        // Record the clone operation
        {
            let mut cloned = self._cloned_repos.write().unwrap();
            cloned.push(ClonedRepo {
                url: url.to_string(),
                path: path.to_path_buf(),
                branch: branch.to_string(),
            });
        }
        
        // Look up registered remote and create files
        let remotes = self._remotes.read().unwrap();
        if remotes.contains_key(url) {
            drop(remotes); // Release lock before creating files
            let remotes = self._remotes.read().unwrap();
            let registration = remotes.get(url).unwrap();
            self.create_mock_bundle_files(path, registration)?;
        } else {
            // Create minimal directory structure for unregistered repos
            fs::create_dir_all(path)?;
            
            let manifest = BundleManifest {
                gitf2_version: "0.1.0".to_string(),
                identifier: GITF2_IDENTIFIER.to_string(),
                description: Some(format!("Mock bundle from {}", url)),
                root: None,
                bundles: HashMap::new(),
            };
            
            let manifest_path = path.join("bundle.toml");
            save_manifest(&manifest, &manifest_path)?;
        }
        
        Ok(())
    }
    
    fn fetch_repository(&self, _path: &Path, _branch: &str) -> Result<()> {
        // Mock: do nothing, consider it fetched
        Ok(())
    }
    
    fn init_repository(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        
        let mut initialized = self._initialized_repos.write().unwrap();
        initialized.push(path.to_path_buf());
        
        Ok(())
    }
    
    fn add_remote(&self, _path: &Path, _name: &str, _url: &str) -> Result<()> {
        // Mock: do nothing
        Ok(())
    }
    
    fn commit_all(&self, _path: &Path, _message: &str) -> Result<()> {
        // Mock: do nothing
        Ok(())
    }
    
    fn push(&self, _path: &Path, _remote: &str, _branch: &str) -> Result<()> {
        // Mock: do nothing
        Ok(())
    }
    
    fn has_local_changes(&self, path: &Path) -> Result<bool> {
        let changes = self._local_changes.read().unwrap();
        Ok(changes.get(path).copied().unwrap_or(false))
    }
    
    fn is_repository(&self, path: &Path) -> bool {
        let initialized = self._initialized_repos.read().unwrap();
        initialized.contains(&path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_git_clone_records_operation() {
        let mock = MockGitOperations::new();
        
        let temp_dir = std::env::temp_dir().join("gitf2_mock_test");
        let _ = fs::remove_dir_all(&temp_dir);
        
        mock.clone_repository(
            "https://github.com/test/repo.git",
            &temp_dir,
            "main",
        ).unwrap();
        
        let cloned = mock.get_cloned_repos();
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned[0].url, "https://github.com/test/repo.git");
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_mock_git_creates_manifest() {
        let mock = MockGitOperations::new();
        
        mock.register_remote_bundle(
            "https://github.com/test/bundle.git",
            "",
            MockBundleContent {
                description: "Test bundle".to_string(),
                files: vec![("test.txt".to_string(), "Hello".to_string())],
            },
        );
        
        let temp_dir = std::env::temp_dir().join("gitf2_mock_manifest_test");
        let _ = fs::remove_dir_all(&temp_dir);
        
        mock.clone_repository(
            "https://github.com/test/bundle.git",
            &temp_dir,
            "main",
        ).unwrap();
        
        assert!(temp_dir.join("bundle.toml").exists());
        assert!(temp_dir.join("test.txt").exists());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
