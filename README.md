# 🔐 IronKey

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.0.2--beta-green.svg)](https://github.com/ronakgh97/ironkeys)

**IronKey** is a fast, simple, and secure command-line password manager that stores your secrets locally using
military-grade encryption (AES-256-GCM).

---

## Features

- **AES-256-GCM encryption** - Military-grade security
- **PBKDF2 key derivation** - 100,000 iterations for password hardening
- **Export/Import** - Backup and restore your vault with triple-password security
- **Entry locking** - Lock sensitive entries for extra protection
- **Clipboard integration** - Copy passwords without displaying them
- **Auto-clear clipboard** - Automatic clipboard clearing after timeout (default 30s)
- **Password generator** - Generate cryptographically secure random passwords
- **Search & Filter** - Find entries quickly with case-insensitive search
- **Simple commands** - Intuitive CLI interface
- **Local storage** - Your data stays on your machine

---

## New Release: v0.0.2-beta

### Export/Import System

- **Backup your vault** - Create encrypted backups with `ik export`
- **Triple-password security** - Separate encryption layer for exports
- **Smart import modes** - Merge, replace, or preview (diff) before importing
- **Default exports folder** - Centralized backup storage at `%APPDATA%\ironkey\exports\`
- **Auto-generated names** - Timestamp-based names for quick backups
- **List backups** - See all your exports with `ik export --list`
- **Full path display** - Always know exactly where files are saved
- **Lock preservation** - Locked entries stay locked across export/import

---

### Installation

#### Using Cargo (recommended) ✘

```bash
rustup update
cargo install ironkey
```

#### Scoop (Windows) ✘

```powershell
scoop bucket add extras
scoop install ironkey
```

#### Homebrew (macOS/Linux) ✘

```bash
brew install ronakgh97/tap/ironkey
```

#### Or build manually using cargo ✓

```bash
# Clone the repository
git clone https://github.com/ronakgh97/ironkeys.git
cd ironkey

# Build manually
rustup update
cargo build --release

# Install using cargo
cargo install --path .
```

---

## Usage

### Commands

| Command                      | Description                                     | Example                                |
|------------------------------|-------------------------------------------------|----------------------------------------|
| `ik`                         | Show welcome screen with status                 | `ik`                                   |
| `ik init`                    | Initialize vault with master password           | `ik init`                              |
| `ik create`                  | Create a new password entry                     | `ik create -k github -v token123`      |
| `ik get`                     | Retrieve a password                             | `ik get -k github`                     |
| `ik get --copy`              | Copy password to clipboard (auto-clears in 30s) | `ik get -k github --copy`              |
| `ik get --copy --timeout 60` | Custom auto-clear timeout                       | `ik get -k github --copy --timeout 60` |
| `ik get --copy --no-clear`   | Copy without auto-clear                         | `ik get -k github --copy --no-clear`   |
| `ik update`                  | Update an existing entry                        | `ik update -k github -v new_token`     |
| `ik list`                    | List all entries                                | `ik list`                              |
| `ik list --search <term>`    | Search entries (case-insensitive)               | `ik list --search "api"`               |
| `ik list --locked`           | Show only locked entries                        | `ik list --locked`                     |
| `ik list --unlocked`         | Show only unlocked entries                      | `ik list --unlocked`                   |
| `ik lock`                    | Lock/unlock an entry                            | `ik lock -k github`                    |
| `ik delete`                  | Delete an entry                                 | `ik delete -k github`                  |
| `ik generate`                | Generate secure random password                 | `ik generate --length 20`              |
| `ik generate --key <name>`   | Generate and save to vault                      | `ik generate -k github --copy`         |
| `ik export`                  | Export vault to encrypted backup                | `ik export --name mybackup`            |
| `ik export --list`           | List all available backups                      | `ik export --list`                     |
| `ik import`                  | Import vault from backup                        | `ik import --name mybackup`            |

### Examples

```bash
# Initialize vault
ik init

# Store passwords
ik create --key "email" --value "mypassword123"
ik create --key "aws" --value "AKIAIOSFODNN7EXAMPLE"
ik create --key "database" --value "super_secret_db_pass"

# Retrieve password (displays on screen)
ik get --key "email"

# Copy password to clipboard (doesn't display)
ik get --key "email" --copy
# Value copied to clipboard! (auto-clearing in 30s)

# Copy with custom timeout (60 seconds)
ik get --key "email" --copy --timeout 60
# Value copied to clipboard! (auto-clearing in 60s)

# Copy without auto-clear (keeps password in clipboard)
ik get --key "email" --copy --no-clear
# Value copied to clipboard!

# Update password
ik update --key "email" --value "new_password456"

# Lock sensitive entry
ik lock --key "database"

# Try to access locked entry (will fail)
ik get --key "database"
# Error: Entry 'database' is locked

# Unlock it
ik lock --key "database"

# List all entries
ik list
# Stored entries:
#   - email
#   - aws
#   - database [LOCKED]

# Search for entries
ik list --search "api"
# Entries matching 'api':
#   - aws_api_key [LOCKED]
#   - github_api
#   - stripe_api_secret

# Show only locked entries
ik list --locked
# Stored entries (locked only):
#   - aws [LOCKED]
#   - database [LOCKED]

# Show only unlocked entries
ik list --unlocked
# Stored entries (unlocked only):
#   - email
#   - github_token

# Combine search and filter
ik list --search "password" --locked
# Entries matching 'password' (locked only):
#   - database_password [LOCKED]

# Delete entry
ik delete --key "email"

# ✓ Generate secure password (16 characters, all types)
ik generate
# ✓ Generated password: aB3$xY9!mN7&qZ2@

# ✓ Generate 24-character password
ik generate --length 24
# ✓ Generated password: kL5#pQ8*rT2@wV6!nM4$gH7&

# Generate and save to vault
ik generate --key "new-api-key"
# Enter master password: ********
# ✓ Generated password saved as 'new-api-key'
# ✓ Generated password: xM9!pQ2#rT5$wV8@

# Generate, save, AND copy to clipboard
ik generate --key "github-token" --copy
# Enter master password: ********
# ✓ Generated password saved as 'github-token'
# ✓ Generated password copied to clipboard! (auto-clearing in 30s)

# Generate PIN (numbers only)
ik generate --length 6 --no-lowercase --no-uppercase --no-symbols
# ✓ Generated password: 837492

# Generate alphanumeric password (no symbols)
ik generate --length 20 --no-symbols
# ✓ Generated password: aB3xY9mN7qZ2wV6knM4p

# Export your vault
ik export --name mybackup
# Enter master password: ********
# Enter export password: ********
# Confirm export password: ********
# ✓ Exported 5 entries to 'C:\Users\...\AppData\Roaming\ironkey\exports\mybackup.ik'

# Export with auto-generated timestamp name
ik export
# ✓ Exported 5 entries to '...\exports\vault_2025-10-04_14-30-45.ik'

# Export to custom location
ik export --output C:/backups/vault_backup
# ✓ Exported 5 entries to 'C:\backups\vault_backup.ik'

# List all backups
ik export --list
# 📦 Available Exports (in C:\Users\...\AppData\Roaming\ironkey\exports):
#
#   1. vault_2025-10-04_14-30-45.ik    (125 KB)  2 hours ago
#   2. mybackup.ik                      (98 KB)   yesterday
#
#   Total: 2 exports (223 KB)

# Import from backup (by name)
ik import --name mybackup
# Enter master password: ********
# Enter import password: ********
# ✓ Import completed successfully!
#   Total entries in export file: 5
#
#   Added 3 new entries:
#     + aws
#     + github
#     + database

# Import from custom location
ik import --input C:/backups/vault_backup.ik

# Preview import without applying (dry-run)
ik import --name mybackup --diff
# Preview (no changes made):
#   Total entries in export file: 5
#
#   Would add 3 new entries:
#     + aws
#     + github
#     + database
#
#   Would skip 2 existing entries:
#     - email
#     - stripe
#
# ✦    Run without --diff to apply changes

# Import with replace mode (overwrites existing)
ik import --name mybackup --replace
# ⚠  WARNING: Replace mode will OVERWRITE existing entries!
# Type 'yes' to confirm: yes
# ✓ Import completed successfully!
#   Updated 2 existing entries:
#     ↻ email
#     ↻ github
```

---

## Export/Import Guide

### Triple-Layer Security

Exports use a **separate encryption layer** for maximum security:

1. **Master Password** - Unlocks your vault
2. **Export Password** - Encrypts the export file
3. **Import Password** - Decrypts the export file (same as export password)
4. **Destination Master** - Unlocks destination vault (can be different!)

### 📤 Export Workflows

```bash
# Quick daily backup (auto-named with timestamp)
ik export
→ Creates: vault_2025-10-04_14-30-45.ik in default folder

# Named backup for easy import
ik export --name weekly_backup
→ Creates: weekly_backup.ik in default folder

# Export to USB drive
ik export --output E:/backups/vault
→ Creates: E:\backups\vault.ik

# Force overwrite existing backup
ik export --name backup --force
→ Overwrites existing backup.ik without error

# Check all your backups
ik export --list
→ Shows all .ik files with sizes and timestamps
```

### 📥 Import Workflows

```bash
# Restore from backup (merge mode - default)
ik import --name weekly_backup
→ Adds new entries, keeps existing ones

# Replace mode (overwrites existing entries)
ik import --name weekly_backup --replace
→ Updates existing entries with imported versions

# Preview before importing (dry-run)
ik import --name weekly_backup --diff
→ Shows what would change without applying

# Import from external file
ik import --input ./shared_vault.ik
→ Imports from custom location
```

### Security Notes

- **Export files are AES-256-GCM encrypted** - Safe to store in cloud (Dropbox, Google Drive)
- **Unique encryption per export** - Same vault + same password = different encrypted output
- **Lock status preserved** - Locked entries stay locked across export/import
- **Use strong export passwords** - Treat export password like master password
- **Test your exports** - Always verify imports work after creating exports

### Default Locations

- **Exports folder**: `%APPDATA%\ironkey\exports\` (Windows) or `~/.config/ironkey/exports/` (Unix)
- **Main database**: `%APPDATA%\ironkey\ironkey.json`

---

## Security

### Encryption

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Derivation**: PBKDF2-HMAC-SHA256 with 100,000 iterations
- **Nonce**: Unique 12-byte random nonce per entry
- **Password Input**: Hidden input using `rpassword` crate

### What's Encrypted?

**Encrypted:**

- All password values
- Entry data

**Not Encrypted:**

- Entry keys (names)
- Lock status flags
- Database structure

### Where is Data Stored?

- **Main Database**:
    - **Windows**: `%APPDATA%\ironkey\ironkey.json`
    - **Linux**: `~/.config/ironkey/ironkey.json`
    - **macOS**: `~/Library/Application Support/ironkey/ironkey.json`

- **Export Backups**:
    - **Windows**: `%APPDATA%\ironkey\exports\`
    - **Linux**: `~/.config/ironkey/exports/`
    - **macOS**: `~/Library/Application Support/ironkey/exports/`

### Security Notes

- **Master password is critical** - If you forget it, your data is **unrecoverable**
- **No backdoors** - Your data is encrypted with your password only
- **Local only** - No cloud, no network, no telemetry
- **Open source** - Audit the code yourself

---

### Key Components

- **Vault** - Manages entries and master key
- **Crypto** - Pure encryption functions (AES-256-GCM)
- **Storage** - JSON database persistence
- **CLI** - User interface via Clap
- **Clipboard** - Clipboard integration and auto-clear
- **Password Generator** - Secure random password generation
- **Export/Import** - Vault backup and migration with triple-password security
- **Entry Locking** - Lock/unlock sensitive entries
- **Search & Filter** - Find entries with case-insensitive search and lock status filtering

---

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Cargo

### Build from Source using `Just` (Recommended for easier life)

[Just Cmds](justfile)

```bash
cargo install just  # One-time install

# Clone repository
git clone https://github.com/ronakgh97/ironkeys.git
cd ironkey

# Build in release mode
just build-dev

# Auto-fix lint issues
just fix

# Run all tests
just test

# Run demo
just version
```

### Dependencies

- `clap` - CLI framework
- `ring` - Cryptographic operations
- `serde` + `serde_json` - JSON serialization
- `base64` - Binary encoding
- `dirs` - Config directory discovery
- `zeroize` - Secure memory cleanup
- `rpassword` - Hidden password input
- `figlet-rs` - ASCII art
- `arboard` - Clipboard integration
- `chrono` - Timestamp handling for exports

---

Current test coverage:

- ✓ Encryption/decryption roundtrip
- ✓ Password verification
- ✓ Nonce uniqueness
- ✓ Database creation
- ✓ Entry serialization
- ✓ Export/Import functionality
- ✓ Round-trip data integrity
- ✓ Clipboard operations
- ✓ Password generation
- ✓ Search and filtering
- ✓ Export and import edge cases

---

## Troubleshooting

### "Database not found" error

**Problem:** You haven't initialized the vault yet.

**Solution:**

```bash
ik init
```

### "Invalid master password" error

**Problem:** Wrong password entered.

**Solution:** Enter the correct master password. If forgotten, your data is unrecoverable.

### "Entry is locked" error

**Problem:** Entry is locked and needs to be unlocked.

**Solution:**

```bash
ik lock --key "entry_name"  # Unlocks it
```

### Find database location

```bash
# Windows PowerShell
echo $env:APPDATA\ironkey\ironkey.json

# Linux/macOS
echo ~/.config/ironkey/ironkey.json
```

---

## Plans

### ✓ Completed

- [x] Core encryption (AES-256-GCM)
- [x] Master password system
- [x] CRUD operations
- [x] Entry locking/unlocking mechanism
- [x] Hidden password input
- [x] Clean architecture
- [x] Clipboard Integration
- [x] Auto-clear Clipboard
- [x] Password Generator
- [x] Search/Filter Entries
- [x] **Export/Import Functionality** - Backup and migration
    - [x] Export entire vault with separate encryption
    - [x] Import with merge/replace/diff modes
    - [x] Default exports directory with auto-naming
    - [x] List all backups command
    - [x] Triple-password security architecture
    - [x] Lock status preservation across export/import

- [ ] **Password Strength Indicator** - Validate master password strength
    - Real-time strength analysis during init
    - Recommendations for weak passwords
    - Block very weak passwords

- [ ] **Session Unlock Mode** - Cache password temporarily
    - `ik unlock` - Unlock vault for 5 minutes
    - No password prompt for list/get during session
    - Auto-lock after timeout or manual `ik lock`
    - Security: Encrypted in-memory storage

- [ ] **Batch Entry Creation** - Create multiple entries efficiently
    - Interactive batch mode: Enter multiple entries in one session
    - Prompt once for master password
    - Add entries until Ctrl+C or empty input
    - UX: `ik create --batch` starts interactive mode

- [ ] **Auto-lock Timeout** - Security improvement
    - Keep vault unlocked for specified duration
    - Auto-clear from memory after timeout
    - Interactive shell mode

- [ ] **Multiple Vault Support** - Separate personal/work passwords
    - Named vaults: `--vault personal` / `--vault work`
    - Set default vault
    - Easy switching between vaults

- [ ] **Tags/Categories** - Better organization
    - Add tags to entries: `--tags "work,api,aws"`
    - Filter by tag: `ik list --tag "work"`
    - Multiple tags per entry

- [ ] **Audit Logging** - Track access history
    - Log all operations (get, create, update, delete)
    - View audit trail: `ik audit`
    - Per-entry history

- [ ] **TOTP/2FA Support** - Store 2FA tokens
    - Add TOTP secrets to entries
    - Generate current TOTP codes
    - Time-based automatic refresh

- [ ] **Database Encryption at Rest** - Full file encryption
    - Encrypt entire JSON file (not just entries)
    - Transparent decryption on load

- [ ] **Cloud Integration (opt-in, end-to-end encrypted, production)**
    - Goals
        - Optional sync across devices
        - Server never sees plaintext or master password (zero-knowledge).
    - Security Model
        - Client-side encryption only: reuse existing AES-256-GCM per-entry.
        - Derive `Sync Key` from master via PBKDF2 for cloud operations.
        - Store entry names as `HMAC-SHA256(name, Sync Key)` to hide metadata.
        - Rotate `Sync Key` via re-derivation when master changes.
    - Architecture
        - Introduce minimal REST service (Rust `axum`) as API facade.
        - Service stores opaque blobs and indexes; all crypto is client-side.
    - Data Model (server-side, all blobs encrypted)
        - `users(id, email_hash, created_at)`
        - `devices(id, user_id, pubkey_fpr, created_at)`
        - `vaults(id, user_id, version, updated_at)`
        - `entries(vault_id, key_hmac, blob, lock_flag, version, updated_at)`
    - API Endpoints
        - `POST /v1/auth/login` → short-lived JWT (email+TOTP).
        - `GET /v1/vault/snapshot` → full encrypted snapshot + version.
        - `POST /v1/vault/sync` → upload deltas, download server deltas.
        - `POST /v1/devices/register` → pair device (optional).
    - CLI Additions
        - `ik account login` \| `logout` \| `status`
        - `ik sync push` \| `pull` \| `run` (two-way)
        - `ik sync resolve --strategy lww` (default last-write-wins) \| `--interactive`
        - Config: token stored securely (Windows Credential Manager) or `%APPDATA%\ironkey\config.json` with OS secret
          storage preferred.
    - Sync Semantics
        - Versioned entries with `updated_at` and monotonic `version`.
        - Default conflict: last-write-wins; optional interactive resolver.
        - Batch, idempotent operations; retry-safe; exponential backoff.
    - Migration/Compatibility
        - No change to on-disk format; cloud uses the same encrypted entry blobs.
        - Local-only remains default; cloud is explicit `ik account login`.
    - Milestones
        - Client E2EE refactor: add `Sync Key`, HMAC of entry names.
        - REST service (auth, snapshot, sync) + Postgres schema.
        - CLI: `account` and `sync` commands with LWW.
        - Device registration + token storage integration (Windows).


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Code Guidelines

- Add tests for new features
- Update documentation
- Use Just for tasks
- Update Just for new commands
- Run `just check` and fix warnings
- Ensure `just test-fast` passes

### Running Tests

**⚠ Important**: Due to clipboard access conflicts, tests must run single-threaded.

**Why `--test-threads=1`?**  
The clipboard tests access the system clipboard, which can only handle one operation at a time. Running tests in
parallel causes `STATUS_HEAP_CORRUPTION` errors on damn Windows.


## ⚠️ Disclaimer

This is a beta version (v0.0.2-beta). While it uses Good-standard encryption, use at your own risk. Always backup
your vault using the export feature.
(Of course, there is no httpclient code that would let me track you, but still be cautious. 🔪)

---