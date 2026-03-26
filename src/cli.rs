use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "blueprint", about = "Cross-agent context injection tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Save a markdown document as a blueprint
    Save {
        /// The handle (name) for this blueprint
        #[arg(long)]
        handle: String,

        /// Path to a markdown file to save
        #[arg(short = 'f', long = "file")]
        file: Option<String>,

        /// Inline markdown content (positional, mutually exclusive with -f)
        #[arg(conflicts_with = "file")]
        content: Option<String>,

        /// Save to global storage (~/.blueprint/) instead of project-level
        #[arg(long)]
        global: bool,
    },

    /// Load a blueprint and resolve all references
    Load {
        /// The handle of the blueprint to load
        handle: String,

        /// Load from global storage
        #[arg(long)]
        global: bool,
    },

    /// Output a self-description for AI agent system prompts
    Skill,

    /// List available blueprints
    List {
        /// List global blueprints instead of project-level
        #[arg(long)]
        global: bool,
    },
}
