//! gitf2 - Git for Files
//!
//! A file package manager that resembles Git and NPM, but for files in general.
//! Manages file bundles using git repositories as the backend storage.

pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod types;

#[cfg(test)]
mod unit_tests;

#[cfg(test)]
mod integration_tests;
