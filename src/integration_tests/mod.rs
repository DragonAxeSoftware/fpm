//! Integration tests for gitf2
//!
//! These tests run with real external dependencies (actual git repositories).
//! They require network access and valid git credentials.
//!
//! Run with: `cargo test --features integration`
//!
//! Note: These tests are ignored by default. Use `#[ignore]` attribute
//! and run with `cargo test -- --ignored` when needed.

// TODO: Add integration tests that interact with real git repositories
// Example test structure:
//
// #[test]
// #[ignore] // Run only when explicitly requested
// fn test_clone_from_real_repository() {
//     // Test cloning from a real public git repository
// }
