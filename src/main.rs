mod cli;
mod commands;
mod error;
mod frontmatter;
mod model;
mod resolver;
mod skill_text;
mod storage;
mod validator;

use clap::Parser;
use cli::{Cli, Command};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    info!(command = ?cli.command, "blueprint invoked");

    let result = match cli.command {
        Command::Save {
            handle,
            file,
            content,
            global,
        } => commands::save::run(&handle, file.as_deref(), content.as_deref(), global).await,
        Command::Load { handle, global } => commands::load::run(&handle, global).await,
        Command::Skill => commands::skill::run(),
        Command::List { global } => commands::list::run(global).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
