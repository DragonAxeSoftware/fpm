use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::types::{BundleManifest, FPM_IDENTIFIER};
use crate::version::check_manifest_compatibility;

/// Loads and parses a bundle.toml manifest file
pub fn load_manifest(path: &Path) -> Result<BundleManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;

    let manifest = parse_manifest(&content)?;

    // Check version compatibility and warn if needed
    check_manifest_compatibility(&manifest.fpm_version);

    Ok(manifest)
}

/// Parses a manifest from TOML string content
pub fn parse_manifest(content: &str) -> Result<BundleManifest> {
    let manifest: BundleManifest =
        toml::from_str(content).context("Failed to parse bundle.toml")?;

    if !manifest.is_valid_fpm_manifest() {
        anyhow::bail!(
            "Invalid fpm manifest: identifier must be '{}', found '{}'",
            FPM_IDENTIFIER,
            manifest.identifier
        );
    }

    Ok(manifest)
}

/// Saves a manifest to a file
pub fn save_manifest(manifest: &BundleManifest, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(manifest).context("Failed to serialize manifest")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write manifest: {}", path.display()))?;

    Ok(())
}

/// Checks if a path contains a valid bundle.toml
pub fn has_manifest(dir: &Path) -> bool {
    let manifest_path = dir.join("bundle.toml");
    if !manifest_path.exists() {
        return false;
    }

    load_manifest(&manifest_path).is_ok()
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::types::BundleDependency;

    #[test]
    fn test_parse_valid_manifest() {
        let content = r#"
            fpm_version = "0.1.0"
            identifier = "fpm-bundle"
            description = "A test bundle"
            
            [bundles.design-from-martha]
            version = "1.0.0"
            git = "https://github.com/example/designs.git"
            path = "martha-designs"
        "#;

        let manifest = parse_manifest(content).unwrap();
        assert_eq!(manifest.fpm_version, "0.1.0");
        assert_eq!(manifest.description, Some("A test bundle".to_string()));
        assert!(manifest.bundles.contains_key("design-from-martha"));
    }

    #[test]
    fn test_parse_invalid_identifier() {
        let content = r#"
            fpm_version = "0.1.0"
            identifier = "wrong-identifier"
        "#;

        let result = parse_manifest(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid fpm manifest"));
    }

    #[test]
    fn test_roundtrip_manifest() {
        let mut manifest = BundleManifest::new("0.1.0");
        manifest.description = Some("Test description".to_string());
        manifest.bundles.insert(
            "test-bundle".to_string(),
            BundleDependency {
                version: "1.0.0".to_string(),
                git: "https://github.com/test/repo.git".to_string(),
                path: None,
                branch: None,
                ssh_key: None,
            },
        );

        let serialized = toml::to_string_pretty(&manifest).unwrap();
        let deserialized = parse_manifest(&serialized).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
