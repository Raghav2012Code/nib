use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "nib",
    version,
    about = "Offline pastebin for your terminal",
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    /// Paste id or prefix, shorthand for `nib get`
    pub id: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Store a paste and print its id
    Add {
        /// Read from this file instead of stdin
        file: Option<PathBuf>,
        /// Expire after a while: 30s, 10m, 24h, 7d
        #[arg(long)]
        ttl: Option<String>,
        /// Seal the paste behind a passphrase
        #[arg(long)]
        secret: bool,
        /// A clear-text name for a sealed paste
        #[arg(long)]
        label: Option<String>,
    },
    /// List recent pastes
    Ls {
        #[arg(short = 'n', long, default_value_t = 20)]
        limit: usize,
        /// Show every paste
        #[arg(long)]
        all: bool,
    },
    /// Print a paste
    Get { id: String },
    /// Copy a paste to the clipboard
    Copy { id: String },
    /// Edit a paste in $EDITOR
    Edit { id: String },
    /// Search paste contents
    Search {
        #[arg(required = true)]
        query: Vec<String>,
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: usize,
    },
    /// Delete a paste
    Rm {
        id: String,
        /// Delete without asking
        #[arg(long)]
        force: bool,
    },
}
