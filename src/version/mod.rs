//! Version compatibility checking for fpm manifests.

use colored::Colorize;

/// The current fpm binary version (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checks if the manifest's fpm_version is compatible with this binary.
///
/// Compatibility rules (semver):
/// - Major version must match (breaking changes)
/// - Minor/patch mismatches are allowed but will warn if manifest is newer
///
/// Returns true if compatible, false otherwise.
pub fn check_manifest_compatibility(manifest_version: &str) -> bool {
    let (compatible, warning) = _check_manifest_compatibility(manifest_version, VERSION);

    if let Some(msg) = warning {
        eprintln!("{}", msg.yellow());
    }

    compatible
}

/// Pure function for testing - returns (compatible, optional_warning_message)
fn _check_manifest_compatibility(
    manifest_version: &str,
    binary_version: &str,
) -> (bool, Option<String>) {
    let manifest_parts = parse_version(manifest_version);
    let binary_parts = parse_version(binary_version);

    let (m_major, m_minor, m_patch) = match manifest_parts {
        Some(v) => v,
        None => {
            return (
                true,
                Some(format!(
                    "Warning: Could not parse manifest fpm_version '{}'. Proceeding anyway.",
                    manifest_version
                )),
            )
        }
    };

    let (b_major, b_minor, b_patch) = match binary_parts {
        Some(v) => v,
        None => return (true, None), // Can't parse binary version, skip check
    };

    // Major version mismatch - incompatible
    if m_major != b_major {
        let msg = format!(
            "Warning: Manifest fpm_version ({}) has different major version than fpm binary ({}). \
            Consider updating the manifest's fpm_version field.",
            manifest_version, binary_version
        );
        return (false, Some(msg));
    }

    // Manifest is newer than binary - warn
    if (m_minor, m_patch) > (b_minor, b_patch) {
        let msg = format!(
            "Warning: Manifest fpm_version ({}) is newer than fpm binary ({}). \
            Some features may not be available. Consider updating fpm.",
            manifest_version, binary_version
        );
        return (true, Some(msg));
    }

    // Binary is newer than manifest - gentle suggestion
    if (b_minor, b_patch) > (m_minor, m_patch) {
        // Only warn for minor version differences, not patch
        if b_minor > m_minor {
            let msg = format!(
                "Note: Manifest fpm_version ({}) is older than fpm binary ({}). \
                Consider updating the manifest's fpm_version field.",
                manifest_version, binary_version
            );
            return (true, Some(msg));
        }
    }

    (true, None)
}

/// Parses a semver string into (major, minor, patch)
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;

    Some((major, minor, patch))
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_parse_version_valid() {
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("0.1"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version("1.x.0"), None);
    }

    #[test]
    fn test_same_version_compatible() {
        let (compatible, warning) = _check_manifest_compatibility("0.1.0", "0.1.0");
        assert!(compatible);
        assert!(warning.is_none());
    }

    #[test]
    fn test_major_version_mismatch_incompatible() {
        let (compatible, warning) = _check_manifest_compatibility("1.0.0", "0.1.0");
        assert!(!compatible);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("different major version"));
    }

    #[test]
    fn test_manifest_newer_minor_warns() {
        let (compatible, warning) = _check_manifest_compatibility("0.2.0", "0.1.0");
        assert!(compatible);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("newer than fpm binary"));
    }

    #[test]
    fn test_binary_newer_minor_suggests_update() {
        let (compatible, warning) = _check_manifest_compatibility("0.1.0", "0.2.0");
        assert!(compatible);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("Consider updating the manifest"));
    }

    #[test]
    fn test_patch_difference_no_warning() {
        // Binary newer by patch only - no warning
        let (compatible, warning) = _check_manifest_compatibility("0.1.0", "0.1.1");
        assert!(compatible);
        assert!(warning.is_none());
    }
}
