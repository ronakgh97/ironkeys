use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ik",
    version = "0.0.2-beta",
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

    /// Creates a new entry
    Create {
        /// Entry name
        #[arg(short, long)]
        key: String,

        /// Value for the entry (if not provided, will prompt securely)
        #[arg(short, long)]
        value: Option<String>,
    },

    /// Gets an entry by name
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

    /// Updates an existing entry
    Update {
        /// Entry name
        #[arg(short, long)]
        key: String,

        /// New value for the entry (if not provided, will prompt securely)
        #[arg(short, long)]
        value: Option<String>,
    },

    /// List all entries with optional search and filter
    List {
        /// Search for entries by name (case-insensitive, partial match)
        #[arg(short, long)]
        search: Option<String>,

        /// Show only locked entries
        #[arg(long, conflicts_with = "unlocked")]
        locked: bool,

        /// Show only unlocked entries
        #[arg(long, conflicts_with = "locked")]
        unlocked: bool,
    },

    /// Deletes an entry
    Delete {
        /// Entry name
        #[arg(short, long)]
        key: String,
    },

    /// Locks an entry (requires master password to unlock)
    Lock {
        /// Entry name
        #[arg(short, long)]
        key: String,
    },

    /// Generates a random secure password
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

        /// Copies to clipboard instead of displaying
        #[arg(short, long, default_value_t = false)]
        copy: bool,

        /// Saves to vault with this key name
        #[arg(short, long)]
        key: Option<String>,
    },

    /// Export vault to encrypted .ik file
    Export {
        /// Custom output path (full path including filename)
        #[arg(short, long, conflicts_with = "name")]
        output: Option<std::path::PathBuf>,

        /// Export name (saved in default exports folder)
        #[arg(short, long, conflicts_with = "output")]
        name: Option<String>,

        /// Force overwrite if file exists
        #[arg(short, long, default_value_t = false)]
        force: bool,

        /// List all available exports in default folder
        #[arg(short, long, default_value_t = false)]
        list: bool,
    },

    /// Import vault from encrypted .ik file
    Import {
        /// Custom input path (full path to .ik file)
        #[arg(short, long, conflicts_with = "name")]
        input: Option<std::path::PathBuf>,

        /// Imports by name (searches default exports folder)
        #[arg(short, long, conflicts_with = "input")]
        name: Option<String>,

        /// Merge: Add new entries, skip existing (default)
        #[arg(short, long, conflicts_with = "replace")]
        merge: bool,

        /// Replace: Overwrite existing entries with imported ones
        #[arg(short, long, conflicts_with = "merge")]
        replace: bool,

        /// Show what would be imported without applying changes (dry-run)
        #[arg(short, long, default_value_t = false)]
        diff: bool,
    },
}
