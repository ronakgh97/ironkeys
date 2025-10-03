use crate::cli::{CliArgs, Commands};
use clap::Parser;
use figlet_rs::FIGfont;

mod cli;
mod clipboard;
mod crypto;
mod error;
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
        Some(Commands::Get { key, copy }) => handle_get(key, copy),
        Some(Commands::Update { key, value }) => handle_update(key, value),
        Some(Commands::List) => handle_list(),
        Some(Commands::Delete { key }) => handle_delete(key),
        Some(Commands::Lock { key }) => handle_lock(key),
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
            println!("\nik init              Initialize your vault");
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

fn handle_get(key: String, copy: bool) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let vault = Vault::unlock(password)?;

    let value = vault.get_entry(&key)?;

    if copy {
        clipboard::copy_to_clipboard(&value)?;
        println!("Value copied to clipboard!");
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

fn handle_list() -> Result<()> {
    let vault = Vault::unlock(prompt_password("Enter master password: ")?)?;

    let entries = vault.list_entries();

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    println!("Stored entries:");
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

fn prompt_password(prompt: &str) -> Result<String> {
    let password = rpassword::prompt_password(prompt)
        .map_err(|e| error::Error::Io(format!("Failed to read password: {e}")))?;

    Ok(password)
}
