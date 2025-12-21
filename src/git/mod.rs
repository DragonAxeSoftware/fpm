use anyhow::{Context, Result};
use git2::{
    build::RepoBuilder, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository,
    RepositoryInitOptions,
};
use std::path::Path;
use tracing::{debug, info};

use crate::types::{BundleDependency, DEFAULT_BRANCH, DEFAULT_REMOTE};

/// Trait for git operations - allows mocking in tests
pub trait GitOperations: Send + Sync {
    fn clone_repository(&self, url: &str, path: &Path, branch: &str) -> Result<()>;
    fn fetch_repository(&self, path: &Path, branch: &str) -> Result<()>;
    fn init_repository(&self, path: &Path) -> Result<()>;
    fn add_remote(&self, path: &Path, name: &str, url: &str) -> Result<()>;
    fn commit_all(&self, path: &Path, message: &str) -> Result<()>;
    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<()>;
    fn has_local_changes(&self, path: &Path) -> Result<bool>;
    fn is_repository(&self, path: &Path) -> bool;
}

/// Default implementation using git2
pub struct Git2Operations;

impl Git2Operations {
    pub fn new() -> Self {
        Self
    }

    fn get_callbacks<'a>() -> RemoteCallbacks<'a> {
        let mut callbacks = RemoteCallbacks::new();
        
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try SSH agent first, then fall back to default credentials
            if let Some(username) = username_from_url {
                Cred::ssh_key_from_agent(username)
            } else {
                Cred::default()
            }
        });
        
        callbacks
    }
}

impl Default for Git2Operations {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOperations for Git2Operations {
    fn clone_repository(&self, url: &str, path: &Path, branch: &str) -> Result<()> {
        info!("Cloning {} to {}", url, path.display());
        
        let callbacks = Self::get_callbacks();
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        RepoBuilder::new()
            .branch(branch)
            .fetch_options(fetch_options)
            .clone(url, path)
            .with_context(|| format!("Failed to clone repository: {}", url))?;
        
        Ok(())
    }

    fn fetch_repository(&self, path: &Path, branch: &str) -> Result<()> {
        debug!("Fetching updates for {}", path.display());
        
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository: {}", path.display()))?;
        
        let mut remote = repo.find_remote("origin")
            .or_else(|_| repo.find_remote(DEFAULT_REMOTE))
            .context("Failed to find remote")?;
        
        let callbacks = Self::get_callbacks();
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&[branch], Some(&mut fetch_options), None)
            .context("Failed to fetch from remote")?;
        
        Ok(())
    }

    fn init_repository(&self, path: &Path) -> Result<()> {
        info!("Initializing git repository at {}", path.display());
        
        let mut opts = RepositoryInitOptions::new();
        opts.initial_head(DEFAULT_BRANCH);
        
        Repository::init_opts(path, &opts)
            .with_context(|| format!("Failed to initialize repository: {}", path.display()))?;
        
        Ok(())
    }

    fn add_remote(&self, path: &Path, name: &str, url: &str) -> Result<()> {
        debug!("Adding remote {} -> {}", name, url);
        
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository: {}", path.display()))?;
        
        // Check if remote already exists
        if repo.find_remote(name).is_ok() {
            debug!("Remote {} already exists, updating URL", name);
            repo.remote_set_url(name, url)?;
        } else {
            repo.remote(name, url)
                .with_context(|| format!("Failed to add remote: {}", name))?;
        }
        
        Ok(())
    }

    fn commit_all(&self, path: &Path, message: &str) -> Result<()> {
        debug!("Committing all changes in {}", path.display());
        
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository: {}", path.display()))?;
        
        // Add all files
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        // Get signature
        let sig = repo.signature()
            .or_else(|_| git2::Signature::now("gitf2", "gitf2@local"))?;
        
        // Get parent commit if exists
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
        
        Ok(())
    }

    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<()> {
        info!("Pushing to {} branch {}", remote, branch);
        
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository: {}", path.display()))?;
        
        let mut remote_obj = repo.find_remote(remote)
            .with_context(|| format!("Remote '{}' not found", remote))?;
        
        let callbacks = Self::get_callbacks();
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote_obj.push(&[&refspec], Some(&mut push_options))
            .with_context(|| format!("Failed to push to {}/{}", remote, branch))?;
        
        Ok(())
    }

    fn has_local_changes(&self, path: &Path) -> Result<bool> {
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open repository: {}", path.display()))?;
        
        let statuses = repo.statuses(None)?;
        
        Ok(!statuses.is_empty())
    }

    fn is_repository(&self, path: &Path) -> bool {
        Repository::open(path).is_ok()
    }
}

/// Clones or updates a bundle from its git source
pub fn fetch_bundle(
    git_ops: &dyn GitOperations,
    dependency: &BundleDependency,
    target_path: &Path,
) -> Result<()> {
    let branch = dependency.branch();
    
    if git_ops.is_repository(target_path) {
        // Repository exists, fetch updates
        git_ops.fetch_repository(target_path, branch)?;
    } else {
        // Clone the repository
        git_ops.clone_repository(&dependency.git, target_path, branch)?;
    }
    
    Ok(())
}

/// Initializes a bundle directory for publishing
pub fn init_bundle_for_publish(
    git_ops: &dyn GitOperations,
    path: &Path,
    remote_url: &str,
) -> Result<()> {
    if !git_ops.is_repository(path) {
        git_ops.init_repository(path)?;
    }
    
    git_ops.add_remote(path, DEFAULT_REMOTE, remote_url)?;
    
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::sync::RwLock;

    struct MockGitOperations {
        cloned_repos: RwLock<Vec<(String, String)>>,
        is_repo: bool,
    }

    impl MockGitOperations {
        fn new(is_repo: bool) -> Self {
            Self {
                cloned_repos: RwLock::new(Vec::new()),
                is_repo,
            }
        }
    }

    impl GitOperations for MockGitOperations {
        fn clone_repository(&self, url: &str, path: &Path, _branch: &str) -> Result<()> {
            self.cloned_repos.write().unwrap().push((
                url.to_string(),
                path.to_string_lossy().to_string(),
            ));
            Ok(())
        }

        fn fetch_repository(&self, _path: &Path, _branch: &str) -> Result<()> {
            Ok(())
        }

        fn init_repository(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn add_remote(&self, _path: &Path, _name: &str, _url: &str) -> Result<()> {
            Ok(())
        }

        fn commit_all(&self, _path: &Path, _message: &str) -> Result<()> {
            Ok(())
        }

        fn push(&self, _path: &Path, _remote: &str, _branch: &str) -> Result<()> {
            Ok(())
        }

        fn has_local_changes(&self, _path: &Path) -> Result<bool> {
            Ok(false)
        }

        fn is_repository(&self, _path: &Path) -> bool {
            self.is_repo
        }
    }

    #[test]
    fn test_fetch_bundle_clones_when_not_exists() {
        let mock = MockGitOperations::new(false);
        let dep = BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/test/repo.git".to_string(),
            path: None,
            branch: None,
        };
        
        let target = Path::new("/tmp/test-bundle");
        fetch_bundle(&mock, &dep, target).unwrap();
        
        let cloned = mock.cloned_repos.read().unwrap();
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned[0].0, "https://github.com/test/repo.git");
    }

    #[test]
    fn test_fetch_bundle_fetches_when_exists() {
        let mock = MockGitOperations::new(true);
        let dep = BundleDependency {
            version: "1.0.0".to_string(),
            git: "https://github.com/test/repo.git".to_string(),
            path: None,
            branch: None,
        };
        
        let target = Path::new("/tmp/test-bundle");
        fetch_bundle(&mock, &dep, target).unwrap();
        
        // Should not clone since repo exists
        let cloned = mock.cloned_repos.read().unwrap();
        assert_eq!(cloned.len(), 0);
    }
}
