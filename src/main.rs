use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fpm::cli::{Cli, Commands};
use fpm::commands::{install, publish, push, status};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install => install::execute(&cli.manifest_path)?,
        Commands::Publish => publish::execute(&cli.manifest_path)?,
        Commands::Push { bundle, message } => {
            push::execute(&cli.manifest_path, bundle.as_deref(), message.as_deref())?
        }
        Commands::Status => status::execute(&cli.manifest_path)?,
    }

    Ok(())
}
