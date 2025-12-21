use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// gitf2 - A file package manager that resembles Git and NPM, but for files in general.
/// 
/// Manages file bundles using git repositories as the backend storage.
#[derive(Parser, Debug)]
#[command(name = "gitf2")]
#[command(author = "DragonAxe Software")]
#[command(version)]
#[command(about = "Git for Files - A file package manager using git as backend")]
#[command(long_about = None)]
pub struct Cli {
    /// Path to the bundle.toml manifest file
    #[arg(short, long, default_value = "bundle.toml")]
    pub manifest_path: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install bundles from the manifest file
    /// 
    /// Fetches all bundles specified in bundle.toml from their git repositories
    /// and places them in .gitf2 subdirectories.
    Install,

    /// Publish bundles to their remote repositories
    /// 
    /// Pushes local bundle changes to the configured git remotes.
    /// Requires version increment if changes have been made.
    Publish,

    /// Push changes in installed bundles back to their source repositories
    /// 
    /// Commits and pushes local modifications made to installed bundles.
    /// Starts from the current manifest and recursively pushes all nested bundles
    /// (deepest first, then parent bundles). Requires write access to the source repositories.
    Push {
        /// Name of a specific bundle to push (pushes all bundles if not specified)
        #[arg(short, long)]
        bundle: Option<String>,

        /// Commit message for the changes
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Show status of all bundles
    /// 
    /// Displays whether bundles are synced, unsynced, or are source bundles.
    Status,
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
