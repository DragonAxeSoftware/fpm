use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// The fpm manifest file identifier
pub const FPM_IDENTIFIER: &str = "fpm-bundle";

/// Default branch name for git operations
pub const DEFAULT_BRANCH: &str = "main";

/// Default remote name for fpm operations
pub const DEFAULT_REMOTE: &str = "fpm";

/// Directory name where bundles are stored
pub const BUNDLE_DIR: &str = ".fpm";

/// The bundle manifest structure (bundle.toml)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundleManifest {
    /// The fpm version that created this manifest
    pub fpm_version: String,

    /// Identifier that marks this as a fpm bundle file
    #[serde(default = "default_identifier")]
    pub identifier: String,

    /// Optional description of what this bundle is about
    #[serde(default)]
    pub description: Option<String>,

    /// Root directory where artifacts are stored (relative to bundle.toml)
    /// If None, this is a purely consuming bundle (assembling-only)
    #[serde(default)]
    pub root: Option<PathBuf>,

    /// List of bundles to fetch
    #[serde(default)]
    pub bundles: HashMap<String, BundleDependency>,
}

fn default_identifier() -> String {
    FPM_IDENTIFIER.to_string()
}

impl BundleManifest {
    pub fn new(fpm_version: &str) -> Self {
        Self {
            fpm_version: fpm_version.to_string(),
            identifier: FPM_IDENTIFIER.to_string(),
            description: None,
            root: None,
            bundles: HashMap::new(),
        }
    }

    pub fn is_valid_fpm_manifest(&self) -> bool {
        self.identifier == FPM_IDENTIFIER
    }

    pub fn is_source_bundle(&self) -> bool {
        self.root.is_some()
    }
}

/// A bundle dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundleDependency {
    /// Version of the bundle to fetch
    pub version: String,

    /// Git repository URL (SSH or HTTPS)
    pub git: String,

    /// Optional subdirectory within the git repository
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Optional branch to fetch from (defaults to "main")
    #[serde(default)]
    pub branch: Option<String>,

    /// Optional path to SSH private key for authentication.
    /// If provided, SSH authentication will be used instead of HTTPS.
    /// The path can be absolute or relative to the user's home directory (e.g., "~/.ssh/id_rsa").
    /// 
    /// NOTE: SSH authentication is implemented but not fully tested yet.
    /// TODO: Add integration tests with SSH key from environment variable.
    #[serde(default)]
    pub ssh_key: Option<PathBuf>,
}

impl BundleDependency {
    pub fn branch(&self) -> &str {
        self.branch.as_deref().unwrap_or(DEFAULT_BRANCH)
    }

    /// Returns true if this dependency should use SSH authentication
    pub fn use_ssh(&self) -> bool {
        self.ssh_key.is_some()
    }
}

/// Status of a bundle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleStatus {
    /// Bundle is synchronized with its remote source
    Synced,
    /// Bundle has local changes or hasn't been downloaded
    Unsynced,
    /// This is a source bundle (has artifacts to publish)
    Source,
}

impl std::fmt::Display for BundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleStatus::Synced => write!(f, "synced"),
            BundleStatus::Unsynced => write!(f, "unsynced"),
            BundleStatus::Source => write!(f, "source"),
        }
    }
}

/// Information about a resolved bundle
#[derive(Debug, Clone)]
pub struct ResolvedBundle {
    /// Name of the bundle
    pub name: String,
    /// Local path where the bundle is/should be stored
    pub local_path: PathBuf,
    /// The dependency specification
    pub dependency: BundleDependency,
    /// Current status
    pub status: BundleStatus,
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_manifest_serialization() {
        let manifest = BundleManifest::new("0.1.0");
        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        
        assert!(toml_str.contains("fpm_version"));
        assert!(toml_str.contains("fpm-bundle"));
    }

    #[test]
    fn test_manifest_deserialization() {
        let toml_str = r#"
            fpm_version = "0.1.0"
            identifier = "fpm-bundle"
            description = "Test bundle"
            
            [bundles.my-bundle]
            version = "1.0.0"
            git = "https://github.com/example/repo.git"
            path = "bundles/my-bundle"
        "#;
        
        let manifest: BundleManifest = toml::from_str(toml_str).unwrap();
        
        assert_eq!(manifest.fpm_version, "0.1.0");
        assert!(manifest.is_valid_fpm_manifest());
        assert_eq!(manifest.description, Some("Test bundle".to_string()));
        assert!(manifest.bundles.contains_key("my-bundle"));
    }

    #[test]
    fn test_bundle_status_display() {
        assert_eq!(format!("{}", BundleStatus::Synced), "synced");
        assert_eq!(format!("{}", BundleStatus::Unsynced), "unsynced");
        assert_eq!(format!("{}", BundleStatus::Source), "source");
    }

    #[test]
    fn test_is_source_bundle() {
        let mut manifest = BundleManifest::new("0.1.0");
        assert!(!manifest.is_source_bundle());
        
        manifest.root = Some(PathBuf::from("artifacts"));
        assert!(manifest.is_source_bundle());
    }
}
