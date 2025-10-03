use crate::cli::{CliArgs, Commands};
use clap::Parser;
use figlet_rs::FIGfont;

mod cli;
mod clipboard;
mod crypto;
mod error;
mod password_generator;
mod storage;
mod vault;

use error::Result;
use vault::Vault;

fn main() {
    let cli = CliArgs::parse();

    let result = match cli.command {
        None => {
            show_welcome();
            Ok(())
        }
        Some(Commands::Init { master }) => handle_init(master),
        Some(Commands::Create { key, value }) => handle_create(key, value),
        Some(Commands::Get {
            key,
            copy,
            no_clear,
            timeout,
        }) => handle_get(key, copy, no_clear, timeout),
        Some(Commands::Update { key, value }) => handle_update(key, value),
        Some(Commands::List {
            search,
            locked,
            unlocked,
        }) => handle_list(search, locked, unlocked),
        Some(Commands::Delete { key }) => handle_delete(key),
        Some(Commands::Lock { key }) => handle_lock(key),
        Some(Commands::Generate {
            length,
            no_lowercase,
            no_uppercase,
            no_numbers,
            no_symbols,
            copy,
            key,
        }) => handle_generate(
            length,
            !no_lowercase,
            !no_uppercase,
            !no_numbers,
            !no_symbols,
            copy,
            key,
        ),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn show_welcome() {
    // Load the standard font
    let standard_font = FIGfont::standard().unwrap();

    // Render the ASCII art
    let figure = standard_font.convert("iron-key");

    if let Some(figure) = figure {
        println!("{figure}");
    } else {
        // Fallback if figlet fails
        println!("╔═══════════════════════════════════════╗");
        println!("║           I R O N K E Y               ║");
        println!("╚═══════════════════════════════════════╝");
    }

    println!("\n🔐 A Rust-based CLI Password Manager");
    println!("   Version: 0.0.1-beta\n");

    // Check if vault is initialized
    match storage::exists() {
        Ok(true) => {
            println!("  Vault is initialized");
            println!("\nAvailable commands:");
        }
        Ok(false) => {
            println!("  Vault not initialized");
            println!("\n   ik init              Initialize your vault");
        }
        Err(_) => {
            println!("\nAvailable commands:");
        }
    }

    println!("   ik create            Create a new entry");
    println!("   ik get               Retrieve an entry");
    println!("   ik update            Update an existing entry");
    println!("   ik list              List all entries");
    println!("   ik delete            Delete an entry");
    println!("   ik lock              Toggle entry lock");
    println!("   ik generate          Generate a secure password");
    println!("\n📖 Documentation: https://github.com/ronakgh97/ironkeys\n");
}

fn handle_init(master_password: Option<String>) -> Result<()> {
    // Check if database already exists
    if storage::exists()? {
        println!("Master key already exists. Please verify your password:");
        let password = match master_password {
            Some(p) => p,
            None => prompt_password("Enter master password: ")?,
        };

        let is_valid = Vault::verify_master_password(password)?;

        if is_valid {
            println!("Master password verified successfully!");
            Ok(())
        } else {
            Err(error::Error::InvalidMasterPassword)
        }
    } else {
        println!("No master key found. Creating a new one...");
        let password = match master_password {
            Some(p) => p,
            None => prompt_password("Enter new master password: ")?,
        };

        if password.trim().is_empty() {
            return Err(error::Error::EmptyPassword);
        }

        let _vault = Vault::init(password)?;
        println!("Master key and database created successfully!");
        Ok(())
    }
}

fn handle_create(key: String, value: String) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let mut vault = Vault::unlock(password)?;

    vault.create_entry(key.clone(), value)?;
    println!("Entry '{key}' created successfully!");

    Ok(())
}

fn handle_get(key: String, copy: bool, no_clear: bool, timeout: u64) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let vault = Vault::unlock(password)?;

    let value = vault.get_entry(&key)?;

    if copy {
        clipboard::copy_to_clipboard(&value)?;

        if no_clear {
            println!("✓ Value copied to clipboard!");
        } else {
            println!("✓ Value copied to clipboard! (auto-clearing in {timeout}s)");

            // Start auto-clear in background
            clipboard::auto_clear_clipboard(&value, std::time::Duration::from_secs(timeout))?;
        }
    } else {
        println!("Value: {value}");
    }

    Ok(())
}

fn handle_update(key: String, value: String) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let mut vault = Vault::unlock(password)?;

    vault.update_entry(key.clone(), value)?;
    println!("Entry '{key}' updated successfully!");

    Ok(())
}

fn handle_list(search: Option<String>, locked: bool, unlocked: bool) -> Result<()> {
    let vault = Vault::unlock(prompt_password("Enter master password: ")?)?;

    // Determine lock filter
    let lock_filter = if locked {
        Some(true) // Show only locked entries
    } else if unlocked {
        Some(false) // Show only unlocked entries
    } else {
        None // Show all entries
    };

    let entries = vault.list_entries(search.as_deref(), lock_filter)?;

    if entries.is_empty() {
        if search.is_some() || locked || unlocked {
            println!("No matching entries found.");
        } else {
            println!("No entries found.");
        }
        return Ok(());
    }

    // Print header
    if let Some(ref search_term) = search {
        print!("Entries matching '{search_term}'");
    } else {
        print!("Stored entries");
    }

    if locked {
        print!(" (locked only)");
    } else if unlocked {
        print!(" (unlocked only)");
    }
    println!(":");

    // Print entries
    for (key, is_locked) in entries {
        let status = if is_locked { " [LOCKED]" } else { "" };
        println!("  - {key}{status}");
    }

    Ok(())
}

fn handle_delete(key: String) -> Result<()> {
    let password = prompt_password("Enter master password to confirm deletion: ")?;
    let mut vault = Vault::unlock(password)?;

    vault.delete_entry(&key)?;
    println!("Entry '{key}' deleted successfully!");

    Ok(())
}

fn handle_lock(key: String) -> Result<()> {
    let password = prompt_password("Enter master password to toggle lock: ")?;
    let mut vault = Vault::unlock(password)?;

    let is_locked = vault.toggle_lock(&key)?;
    let status = if is_locked { "locked" } else { "unlocked" };
    println!("Entry '{key}' {status} successfully!");

    Ok(())
}

fn handle_generate(
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_numbers: bool,
    use_symbols: bool,
    copy: bool,
    key: Option<String>,
) -> Result<()> {
    // Generate password
    let password = password_generator::generate(
        length,
        use_lowercase,
        use_uppercase,
        use_numbers,
        use_symbols,
    )?;

    // If key option is specified, save to vault
    if let Some(key_name) = key {
        let master_password = prompt_password("Enter master password: ")?;
        let mut vault = Vault::unlock(master_password)?;
        vault.create_entry(key_name.clone(), password.clone())?;
        println!("✓ Generated password saved as '{key_name}'");
    }

    // Handle display/clipboard
    if copy {
        clipboard::copy_to_clipboard(&password)?;
        println!("✓ Generated password copied to clipboard! (auto-clearing in 30s)");
        clipboard::auto_clear_clipboard(&password, std::time::Duration::from_secs(30))?;
    } else {
        println!("Generated password: {password}");
    }

    Ok(())
}

fn prompt_password(prompt: &str) -> Result<String> {
    let password = rpassword::prompt_password(prompt)
        .map_err(|e| error::Error::Io(format!("Failed to read password: {e}")))?;

    Ok(password)
}
