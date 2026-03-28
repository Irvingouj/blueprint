use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "blueprint",
    about = "Cross-agent context injection tool",
    long_about = "Blueprint — Cross-Agent Context Injection Tool\n\
        \n\
        Save markdown documents with file/URL/git-diff references, then load them \
        with all referenced content resolved and inlined.\n\
        \n\
        Commands:\n\
        \n  \
        save   Save a blueprint with references to external resources\n  \
        load   Load a blueprint and resolve all references inline\n  \
        list   List available blueprints with filtering and sorting\n  \
        skill  Output a SKILL.md for AI agent system prompts"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Save a markdown document as a named blueprint
    ///
    /// Reads markdown from a file or inline content, parses YAML frontmatter,
    /// injects base_dir and saved_at metadata, validates references, and writes
    /// to .blueprint/<handle>.md (project-level) or ~/.blueprint/<handle>.md (global).
    Save {
        /// The handle (name) for this blueprint
        #[arg(long)]
        handle: String,

        /// One-line description (auto-extracted from first H1 heading if not provided)
        #[arg(long)]
        description: Option<String>,

        /// Path to a markdown file to save
        #[arg(short = 'f', long = "file")]
        file: Option<String>,

        /// Inline markdown content (mutually exclusive with --file)
        #[arg(conflicts_with = "file")]
        content: Option<String>,

        /// Save to global storage (~/.blueprint/) instead of project-level
        #[arg(long)]
        global: bool,
    },

    /// Load a blueprint and resolve all references inline
    ///
    /// Reads the blueprint file, outputs the markdown body, then resolves all
    /// references (files, URLs, git diffs) concurrently and appends their contents
    /// under a "Referenced Files" section. Failed resolutions produce
    /// [unresolved: <error>] placeholders.
    Load {
        /// The handle of the blueprint to load
        handle: String,

        /// Load from global storage
        #[arg(long)]
        global: bool,

        /// Output only the markdown body without resolving references
        #[arg(long)]
        no_expand: bool,
    },

    /// Output a SKILL.md for AI agent system prompts
    ///
    /// Prints a self-describing markdown document with YAML frontmatter (name,
    /// description) and full documentation of commands, reference types, and usage.
    /// Designed to be placed in .claude/skills/ for automatic agent integration.
    Skill,

    /// List available blueprints with optional sorting and filtering
    ///
    /// Displays a table of blueprints with handle, time-ago, and description columns.
    /// Supports fuzzy matching on handle/description and sorting by creation time.
    List {
        /// List global blueprints instead of project-level
        #[arg(long)]
        global: bool,

        /// Sort by creation time (newest first, blueprints without timestamps go last)
        #[arg(long, short = 't')]
        sort_time: bool,

        /// Fuzzy-match filter on handle or description
        #[arg(short = 'f', long)]
        filter: Option<String>,
    },
}
