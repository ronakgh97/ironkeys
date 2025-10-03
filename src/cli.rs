use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ik",
    version = "0.0.1-beta",
    about = "Iron Key - A Rust-based secret key manager",
    long_about = "IronKey is a lightweight CLI tool for securely managing secret keys."
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new master key
    Init {
        /// Master password
        #[arg(short, long)]
        master: Option<String>,
    },

    /// Create a new entry
    Create {
        /// Entry name
        #[arg(short, long)]
        key: String,

        /// Value for the entry
        #[arg(short, long)]
        value: String,
    },

    /// Get an entry by name
    Get {
        /// Entry name
        #[arg(short, long)]
        key: String,

        /// Copy to clipboard instead of displaying
        #[arg(short, long, default_value_t = false)]
        copy: bool,
    },

    /// Update an existing entry
    Update {
        /// Entry name
        #[arg(short, long)]
        key: String,

        /// New value for the entry
        #[arg(short, long)]
        value: String,
    },

    /// List all entries
    List,

    /// Delete an entry
    Delete {
        /// Entry name
        #[arg(short, long)]
        key: String,
    },

    /// Lock an entry (requires master password to unlock)
    Lock {
        /// Entry name
        #[arg(short, long)]
        key: String,
    },
}
