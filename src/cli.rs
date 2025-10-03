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

        /// Disable auto-clear of clipboard (only with --copy)
        #[arg(long, default_value_t = false)]
        no_clear: bool,

        /// Timeout in seconds before auto-clearing clipboard (default: 30)
        #[arg(short, long, default_value_t = 30)]
        timeout: u64,
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

    /// Generate a random secure password
    Generate {
        /// Length of password (default: 16)
        #[arg(short, long, default_value_t = 16)]
        length: usize,

        /// Exclude lowercase letters (a-z)
        #[arg(long, default_value_t = false)]
        no_lowercase: bool,

        /// Exclude uppercase letters (A-Z)
        #[arg(long, default_value_t = false)]
        no_uppercase: bool,

        /// Exclude numbers (0-9)
        #[arg(long, default_value_t = false)]
        no_numbers: bool,

        /// Exclude symbols (!@#$%^&*()_+-=[]{}|;:,.<>?)
        #[arg(long, default_value_t = false)]
        no_symbols: bool,

        /// Copy to clipboard instead of displaying
        #[arg(short, long, default_value_t = false)]
        copy: bool,

        /// Save to vault with this key name
        #[arg(short, long)]
        key: Option<String>,
    },
}
