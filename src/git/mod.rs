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
    fn clone_repository(
        &self,
        url: &str,
        path: &Path,
        branch: &str,
        ssh_key: Option<&Path>,
    ) -> Result<()>;
    fn fetch_repository(&self, path: &Path, branch: &str) -> Result<()>;
    fn init_repository(&self, path: &Path) -> Result<()>;
    fn add_remote(&self, path: &Path, name: &str, url: &str) -> Result<()>;
    fn commit_all(&self, path: &Path, message: &str) -> Result<()>;
    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<()>;
    fn has_local_changes(&self, path: &Path) -> Result<bool>;
    fn is_repository(&self, path: &Path) -> bool;
    /// Get file content from HEAD commit
    fn get_file_from_head(&self, repo_path: &Path, file_path: &str) -> Result<String>;
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
    fn clone_repository(
        &self,
        url: &str,
        path: &Path,
        branch: &str,
        _ssh_key: Option<&Path>,
    ) -> Result<()> {
        // Note: Git2Operations currently ignores ssh_key parameter.
        // For SSH support with custom keys, use GitCliOperations instead.
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
            .or_else(|_| git2::Signature::now("fpm", "fpm@local"))?;
        
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

    fn get_file_from_head(&self, repo_path: &Path, file_path: &str) -> Result<String> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open repository: {}", repo_path.display()))?;
        
        let head = repo.head()
            .context("Failed to get HEAD reference")?;
        let commit = head.peel_to_commit()
            .context("Failed to get HEAD commit")?;
        let tree = commit.tree()
            .context("Failed to get commit tree")?;
        
        let entry = tree.get_path(std::path::Path::new(file_path))
            .with_context(|| format!("File '{}' not found in HEAD", file_path))?;
        
        let blob = repo.find_blob(entry.id())
            .context("Failed to get file blob")?;
        
        let content = std::str::from_utf8(blob.content())
            .context("File content is not valid UTF-8")?;
        
        Ok(content.to_string())
    }
}

/// CLI-based git implementation using the system git command.
/// This is more reliable for HTTPS authentication as it uses the user's
/// configured credential helpers.
pub struct GitCliOperations;

impl GitCliOperations {
    pub fn new() -> Self {
        Self
    }

    fn run_git(&self, args: &[&str], working_dir: Option<&Path>) -> Result<()> {
        self.run_git_with_ssh_key(args, working_dir, None)
    }

    /// Runs a git command with optional SSH key authentication.
    /// When ssh_key is provided, sets GIT_SSH_COMMAND to use the specified key.
    fn run_git_with_ssh_key(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
        ssh_key: Option<&Path>,
    ) -> Result<()> {
        let mut cmd = std::process::Command::new("git");
        cmd.args(args);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        // Set SSH command if an SSH key is provided
        if let Some(key_path) = ssh_key {
            let key_path_str = key_path.to_string_lossy();
            // Use -o StrictHostKeyChecking=accept-new to auto-accept new host keys
            let ssh_command = format!(
                "ssh -i \"{}\" -o StrictHostKeyChecking=accept-new -o BatchMode=yes",
                key_path_str
            );
            cmd.env("GIT_SSH_COMMAND", ssh_command);
            debug!("Using SSH key: {}", key_path_str);
        }
        
        let output = cmd.output()
            .context("Failed to execute git command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git command failed: {}", stderr);
        }
        
        Ok(())
    }
}

impl Default for GitCliOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOperations for GitCliOperations {
    fn clone_repository(
        &self,
        url: &str,
        path: &Path,
        branch: &str,
        ssh_key: Option<&Path>,
    ) -> Result<()> {
        info!("Cloning {} to {} (branch: {})", url, path.display(), branch);
        
        let args = [
            "clone",
            "--branch", branch,
            "--single-branch",
            url,
            &path.to_string_lossy(),
        ];
        
        self.run_git_with_ssh_key(&args, None, ssh_key)
            .with_context(|| format!("Failed to clone repository: {}", url))
    }

    fn fetch_repository(&self, path: &Path, branch: &str) -> Result<()> {
        debug!("Fetching updates for {}", path.display());
        
        self.run_git(&["fetch", "origin", branch], Some(path))
            .context("Failed to fetch from remote")?;
        
        // Reset to the fetched branch
        self.run_git(&["reset", "--hard", &format!("origin/{}", branch)], Some(path))
            .context("Failed to reset to fetched branch")?;
        
        Ok(())
    }

    fn init_repository(&self, path: &Path) -> Result<()> {
        info!("Initializing git repository at {}", path.display());
        
        std::fs::create_dir_all(path)?;
        self.run_git(&["init", "-b", DEFAULT_BRANCH], Some(path))
            .with_context(|| format!("Failed to initialize repository: {}", path.display()))
    }

    fn add_remote(&self, path: &Path, name: &str, url: &str) -> Result<()> {
        debug!("Adding remote {} -> {}", name, url);
        
        // Try to add, if it fails try to set-url
        if self.run_git(&["remote", "add", name, url], Some(path)).is_err() {
            self.run_git(&["remote", "set-url", name, url], Some(path))?;
        }
        
        Ok(())
    }

    fn commit_all(&self, path: &Path, message: &str) -> Result<()> {
        debug!("Committing all changes in {}", path.display());
        
        self.run_git(&["add", "-A"], Some(path))?;
        self.run_git(&["commit", "-m", message], Some(path))?;
        
        Ok(())
    }

    fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<()> {
        info!("Pushing to {} branch {}", remote, branch);
        
        self.run_git(&["push", "-u", remote, branch], Some(path))
            .with_context(|| format!("Failed to push to {}/{}", remote, branch))
    }

    fn has_local_changes(&self, path: &Path) -> Result<bool> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .context("Failed to check git status")?;
        
        Ok(!output.stdout.is_empty())
    }

    fn is_repository(&self, path: &Path) -> bool {
        path.join(".git").exists()
    }

    fn get_file_from_head(&self, repo_path: &Path, file_path: &str) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["show", &format!("HEAD:{}", file_path)])
            .current_dir(repo_path)
            .output()
            .context("Failed to run git show")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get file from HEAD: {}", stderr);
        }
        
        let content = String::from_utf8(output.stdout)
            .context("File content is not valid UTF-8")?;
        
        Ok(content)
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
        let ssh_key = dependency.ssh_key.as_deref();
        git_ops.clone_repository(&dependency.git, target_path, branch, ssh_key)?;
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
        fn clone_repository(
            &self,
            url: &str,
            path: &Path,
            _branch: &str,
            _ssh_key: Option<&Path>,
        ) -> Result<()> {
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

        fn get_file_from_head(&self, _repo_path: &Path, _file_path: &str) -> Result<String> {
            // Mock: return empty string (will cause version comparison to fail gracefully)
            anyhow::bail!("Mock: no HEAD commit")
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
            ssh_key: None,
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
            ssh_key: None,
        };
        
        let target = Path::new("/tmp/test-bundle");
        fetch_bundle(&mock, &dep, target).unwrap();
        
        // Should not clone since repo exists
        let cloned = mock.cloned_repos.read().unwrap();
        assert_eq!(cloned.len(), 0);
    }
}
