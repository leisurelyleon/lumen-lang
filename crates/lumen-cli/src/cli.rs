//! Command-line argument definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// The Lumen language: a tree-walking interpreter.
#[derive(Debug, Parser)]
#[command(name = "lumen", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run a Lumen source file.
    Run {
        /// Path to a `.lum` source file.
        path: PathBuf,
    },
    /// Start an interactive REPL.
    Repl,
}
