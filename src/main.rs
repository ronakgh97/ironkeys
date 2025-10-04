use crate::cli::{CliArgs, Commands};
use clap::Parser;
use figlet_rs::FIGfont;
use std::path::Path;

mod cli;
mod clipboard;
mod crypto;
mod error;
mod export;
mod import;
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
        Some(Commands::Export {
            output,
            name,
            force,
            list,
        }) => handle_export(output, name, force, list),
        Some(Commands::Import {
            input,
            name,
            merge,
            replace,
            diff,
        }) => handle_import(input, name, merge, replace, diff),
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
    println!("   Version: 0.0.2-beta\n");

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

/// Get the default exports directory path
fn get_exports_directory() -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| error::Error::Io("Could not determine config directory".to_string()))?;

    let exports_dir = config_dir.join("ironkey").join("exports");
    Ok(exports_dir)
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
        println!("\n⚠ IMPORTANT SECURITY WARNING:");
        println!("   • There is NO password recovery mechanism! for now");
        println!("   • If you forget your master password, your vault is permanently locked.");
        println!("   • Keep your password safe and consider exporting backups.\n");

        let password = match master_password {
            Some(p) => p,
            None => prompt_password("Enter new master password: ")?,
        };

        if password.trim().is_empty() {
            return Err(error::Error::EmptyPassword);
        }

        let _vault = Vault::init(password)?;
        println!("\n✓ Master key and database created successfully!");
        Ok(())
    }
}

fn handle_create(key: String, value: Option<String>) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let mut vault = Vault::unlock(password)?;

    // If value not provided via CLI, prompt securely
    let entry_value = match value {
        Some(v) => v,
        None => {
            println!("      Value will be hidden");
            prompt_password("Enter value: ")?
        }
    };

    vault.create_entry(key.clone(), entry_value)?;
    println!("✓ Entry '{key}' created successfully!");

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

fn handle_update(key: String, value: Option<String>) -> Result<()> {
    let password = prompt_password("Enter master password: ")?;
    let mut vault = Vault::unlock(password)?;

    // If value not provided via CLI, prompt securely
    let new_value = match value {
        Some(v) => v,
        None => {
            println!("      Value will be hidden");
            prompt_password("Enter new value: ")?
        }
    };

    vault.update_entry(key.clone(), new_value)?;
    println!("✓ Entry '{key}' updated successfully!");

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
            println!("✘ No matching entries found.");
        } else {
            println!("✘ No entries found.");
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

fn handle_export(
    output: Option<std::path::PathBuf>,
    name: Option<String>,
    force: bool,
    list: bool,
) -> Result<()> {
    // Handle --list flag
    if list {
        return list_exports();
    }

    // Resolve output path based on flags
    let output = match (output, name) {
        (None, None) => {
            // No flags: default location with auto-generated timestamp name
            let exports_dir = get_exports_directory()?;
            std::fs::create_dir_all(&exports_dir)?;

            let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
            exports_dir.join(format!("vault_{timestamp}.ik"))
        }
        (None, Some(n)) => {
            // Only --name: use default exports folder
            let exports_dir = get_exports_directory()?;
            std::fs::create_dir_all(&exports_dir)?;

            let mut path = exports_dir.join(&n);
            // Auto-append .ik if missing
            if path.extension().and_then(|s| s.to_str()) != Some("ik") {
                path.set_extension("ik");
            }
            path
        }
        (Some(path), None) => {
            // Only --output: use custom path
            let mut output_path = path;
            // Auto-append .ik if missing
            if output_path.extension().and_then(|s| s.to_str()) != Some("ik") {
                output_path.set_extension("ik");
            }
            output_path
        }
        (Some(_), Some(_)) => {
            // Both flags: this should be prevented by clap's conflicts_with
            unreachable!("clap should prevent using both --output and --name");
        }
    };

    // Prompt for master password
    let master_password = prompt_password("Enter master password: ")?;
    let vault = Vault::unlock(master_password)?;

    // Prompt for export password (with confirmation)
    let export_password = prompt_password("Enter export password: ")?;
    let export_password_confirm = prompt_password("Confirm export password: ")?;

    if export_password != export_password_confirm {
        return Err(error::Error::Io(
            "✘ Export passwords do not match".to_string(),
        ));
    }

    // Export the vault
    if force {
        vault.export_to_file_force(&output, export_password)?;
    } else {
        vault.export_to_file(&output, export_password)?;
    }

    // Count entries by listing them (no filter)
    let entry_count = vault.list_entries(None, None)?.len();

    // Format path to hide username for default exports directory
    let display_path = format_export_path(&output)?;

    println!(
        "✓ Exported {} {} to '{}'",
        entry_count,
        if entry_count == 1 { "entry" } else { "entries" },
        display_path
    );

    Ok(())
}

/// Format export path to hide username in default exports directory
fn format_export_path(path: &Path) -> Result<String> {
    let exports_dir = get_exports_directory()?;

    // If path is in default exports directory, use relative notation
    if let Ok(relative) = path.strip_prefix(&exports_dir) {
        Ok(format!("<exports>/{}", relative.display()))
    } else {
        // Custom path: show full canonical path
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Ok(canonical.display().to_string())
    }
}

fn list_exports() -> Result<()> {
    let exports_dir = get_exports_directory()?;

    if !exports_dir.exists() {
        println!("✘ No exports found.");
        println!("\n✦    Run 'ik export' to create your first backup!");
        return Ok(());
    }

    // Collect all .ik files
    let mut exports = Vec::new();
    for entry in std::fs::read_dir(&exports_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("ik") {
            let metadata = std::fs::metadata(&path)?;
            let size = metadata.len();
            let modified = metadata.modified()?;
            exports.push((path, size, modified));
        }
    }

    if exports.is_empty() {
        println!("✘ No exports found in {}", exports_dir.display());
        println!("\n✦    Run 'ik export --name mybackup' to create a backup!");
        return Ok(());
    }

    // Sort by modification time (newest first)
    exports.sort_by(|a, b| b.2.cmp(&a.2));

    println!("\n📦 Available Exports (in {}):\n", exports_dir.display());

    let total_size: u64 = exports.iter().map(|(_, s, _)| s).sum();

    for (i, (path, size, modified)) in exports.iter().enumerate() {
        let filename = path.file_name().unwrap().to_string_lossy();

        // Format size appropriately
        let size_str = if *size < 1024 {
            format!("{size:>6} B")
        } else {
            format!("{:>6} KB", size / 1024)
        };

        // Format time ago
        let duration = std::time::SystemTime::now()
            .duration_since(*modified)
            .unwrap_or_default();
        let time_ago = format_time_ago(duration);

        println!("  {}. {:<45} ({})  {}", i + 1, filename, size_str, time_ago);
    }

    // Format total size
    let total_size_str = if total_size < 1024 {
        format!("{total_size} B")
    } else {
        format!("{} KB", total_size / 1024)
    };

    println!(
        "\n  Total: {} {} ({})\n",
        exports.len(),
        if exports.len() == 1 {
            "export"
        } else {
            "exports"
        },
        total_size_str
    );

    println!("✦    Use 'ik import --name <filename>' to restore a backup");

    Ok(())
}

fn format_time_ago(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} {}", mins, if mins == 1 { "minute" } else { "minutes" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} {}", hours, if hours == 1 { "hour" } else { "hours" })
    } else if secs < 604800 {
        let days = secs / 86400;
        format!("{} {}", days, if days == 1 { "day" } else { "days" })
    } else if secs < 2592000 {
        let weeks = secs / 604800;
        format!("{} {}", weeks, if weeks == 1 { "week" } else { "weeks" })
    } else if secs < 31536000 {
        let months = secs / 2592000;
        format!(
            "{} {}",
            months,
            if months == 1 { "month" } else { "months" }
        )
    } else {
        let years = secs / 31536000;
        format!("{} {}", years, if years == 1 { "year" } else { "years" })
    }
}

fn handle_import(
    input: Option<std::path::PathBuf>,
    name: Option<String>,
    _merge: bool,
    replace: bool,
    diff: bool,
) -> Result<()> {
    // Resolve input path based on flags
    let input = match (input, name) {
        (None, None) => {
            return Err(error::Error::Io(
                "✘ Must specify either --input or --name".to_string(),
            ));
        }
        (None, Some(n)) => {
            // Only --name: search in default exports folder
            let exports_dir = get_exports_directory()?;
            let mut path = exports_dir.join(&n);

            // Auto-append .ik if missing
            if path.extension().and_then(|s| s.to_str()) != Some("ik") {
                path.set_extension("ik");
            }
            path
        }
        (Some(path), None) => {
            // Only --input: use custom path
            path
        }
        (Some(_), Some(_)) => {
            // Both flags: this should be prevented by clap's conflicts_with
            unreachable!("clap should prevent using both --input and --name");
        }
    };

    // Verify file exists
    if !input.exists() {
        return Err(error::Error::Io(format!(
            "✘ Import file not found: {}",
            input.display()
        )));
    }

    // Validate .ik extension
    if input.extension().and_then(|s| s.to_str()) != Some("ik") {
        return Err(error::Error::Io(format!(
            "✘ Invalid file format: '{}'. Expected .ik file.",
            input.display()
        )));
    }

    // Prompt for master password
    let master_password = prompt_password("Enter master password: ")?;
    let mut vault = Vault::unlock(master_password)?;

    // Prompt for import password
    let import_password = prompt_password("Enter import password: ")?;

    // Determine strategy (default to merge if none specified)
    let (merge_mode, replace_mode, diff_mode) = if diff {
        (false, false, true)
    } else if replace {
        (false, true, false)
    } else {
        (true, false, false) // default: merge
    };

    // Confirm replace mode (destructive operation)
    if replace_mode {
        println!("⚠   WARNING: Replace mode will OVERWRITE existing entries!");
        let confirm = prompt_password("Type 'yes' to confirm: ")?;
        if confirm.to_lowercase() != "yes" {
            println!("Import cancelled.");
            return Ok(());
        }
    }

    // Import the vault
    let result =
        vault.import_from_file(&input, import_password, merge_mode, replace_mode, diff_mode)?;

    // Display results
    if diff_mode {
        println!("  Preview (no changes made):");
        println!("  Total entries in export file: {}", result.total_in_export);
        println!(
            "\n  Would add {} new {}",
            result.added.len(),
            if result.added.len() == 1 {
                "entry"
            } else {
                "entries"
            }
        );
        if !result.added.is_empty() {
            for key in &result.added {
                println!("    + {key}");
            }
        }

        if replace_mode {
            println!(
                "\n  Would update {} existing {}",
                result.updated.len(),
                if result.updated.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            );
            if !result.updated.is_empty() {
                for key in &result.updated {
                    println!("    ↻ {key}");
                }
            }
        } else {
            println!(
                "\n  Would skip {} existing {}",
                result.skipped.len(),
                if result.skipped.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            );
            if !result.skipped.is_empty() {
                for key in &result.skipped {
                    println!("    - {key}");
                }
            }
        }

        println!("\n✦    Run without --diff to apply changes");
    } else {
        // Actual import completed
        println!("✓ Import completed successfully!");
        println!("  Total entries in export file: {}", result.total_in_export);

        if !result.added.is_empty() {
            println!(
                "\n  Added {} new {}:",
                result.added.len(),
                if result.added.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            );
            for key in &result.added {
                println!("    + {key}");
            }
        }

        if !result.updated.is_empty() {
            println!(
                "\n  Updated {} existing {}:",
                result.updated.len(),
                if result.updated.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            );
            for key in &result.updated {
                println!("    ↻ {key}");
            }
        }

        if !result.skipped.is_empty() {
            println!(
                "\n  Skipped {} existing {} (merge mode):",
                result.skipped.len(),
                if result.skipped.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            );
            for key in &result.skipped {
                println!("    - {key}");
            }
        }
    }

    Ok(())
}

fn prompt_password(prompt: &str) -> Result<String> {
    let password = rpassword::prompt_password(prompt)
        .map_err(|e| error::Error::Io(format!("✘ Failed to read password: {e}")))?;

    Ok(password)
}
