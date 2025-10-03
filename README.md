#   🔐 IronKey

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.0.1--beta-green.svg)](https://github.com/ronakgh97/ironkeys)

**IronKey** is a fast, simple, and secure command-line password manager that stores your secrets locally using military-grade encryption (AES-256-GCM).

---

##  Features

- **AES-256-GCM encryption** - Military-grade security
- **PBKDF2 key derivation** - 100,000 iterations for password hardening
- **Entry locking** - Lock sensitive entries for extra protection
- **Clipboard integration** - Copy passwords without displaying them
- **Auto-clear clipboard** - Automatic clipboard clearing after timeout (default 30s)
- **Simple commands** - Intuitive CLI interface
- **Local storage** - Your data stays on your machine

---

### Installation (build from source)

```bash
# Clone the repository
git clone https://github.com/ronakgh97/ironkeys.git
cd ironkey

# Build manually
cargo build --release

# Install using cargo
cargo install --path .
```

### First Time Setup

```bash
# Initialize your vault
ik init
# Enter new master password: ********

# Create your first entry
ik create --key "github" --value "ghp_your_token_here"

# Retrieve it
ik get --key "github"
# Value: ghp_your_token_here
```

---

## Usage

### Commands

| Command | Description | Example |
|---------|-------------|---------|
| `ik` | Show welcome screen with status | `ik` |
| `ik init` | Initialize vault with master password | `ik init` |
| `ik create` | Create a new password entry | `ik create -k github -v token123` |
| `ik get` | Retrieve a password | `ik get -k github` |
| `ik get --copy` | Copy password to clipboard (auto-clears in 30s) | `ik get -k github --copy` |
| `ik get --copy --timeout 60` | Custom auto-clear timeout | `ik get -k github --copy --timeout 60` |
| `ik get --copy --no-clear` | Copy without auto-clear | `ik get -k github --copy --no-clear` |
| `ik update` | Update an existing entry | `ik update -k github -v new_token` |
| `ik list` | List all entries | `ik list` |
| `ik lock` | Lock/unlock an entry | `ik lock -k github` |
| `ik delete` | Delete an entry | `ik delete -k github` |

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

# Delete entry
ik delete --key "email"
```

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

- **Windows**: `%APPDATA%\ironkey\ironkey.json`
- **Linux**: `~/.config/ironkey/ironkey.json`
- **macOS**: `~/Library/Application Support/ironkey/ironkey.json`

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

For detailed code walkthrough, see [`WORKFLOW.md`](WORKFLOW.md).

---

## 🛠️ Development

### Prerequisites

- Rust 1.70+ (Edition 2024)
- Cargo

### Build from Source

```bash
# Clone repository
git clone https://github.com/ronakgh97/ironkeys.git
cd ironkey

# Run in development mode
cargo run

# Build release version
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
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

---

## Testing

```bash
# Run all tests
cargo test
```

Current test coverage:
- ✅ Encryption/decryption roundtrip
- ✅ Password verification
- ✅ Nonce uniqueness
- ✅ Database creation
- ✅ Entry serialization

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

### ✅ Completed
- [x] Core encryption (AES-256-GCM)
- [x] Master password system
- [x] CRUD operations (Create, Read, Update, Delete)
- [x] Entry locking mechanism
- [x] Hidden password input
- [x] Clean architecture
- [x] Clipboard Integration
- [x] Auto-clear Clipboard

- [ ] **Password Generator** - Generate strong random passwords
  - `ik generate --length 20 --symbols`
  - Customizable: length, character sets
  - Option to save directly to vault
  
- [ ] **Search/Filter Entries** - Find entries quickly
  - `ik list --search "github"`
  - Filter by lock status: `--locked` / `--unlocked`
  - Case-insensitive matching

- [ ] **Export/Import Functionality** - Backup and migration
  - Export entire vault or specific entries
  - Import with merge/replace options
  - Encrypted export format
  
- [ ] **Password Strength Indicator** - Validate master password strength
  - Real-time strength analysis during init
  - Recommendations for weak passwords
  - Block very weak passwords
  
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

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Code Guidelines

- Follow Rust best practices
- Add tests for new features
- Update documentation
- Run `cargo clippy` and fix warnings
- Ensure `cargo test` passes

### Running Tests

**⚠️ Important**: Due to clipboard access conflicts, tests must run single-threaded.

#### Using Just (Recommended)

```bash
# Install just (one time)
cargo install just

# Run all tests
just test

# Run specific test file
just test-file clipboard_tests

# Run fast tests (non-clipboard, parallel)
just test-fast

# Run all CI checks
just ci

# See all available commands
just --list
```

#### Using Cargo Directly

```bash
# Run all tests
cargo test -- --test-threads=1

# Run specific test file
cargo test --test clipboard_tests -- --test-threads=1

# Run non-clipboard tests (parallel is fine)
cargo test --test crypto_tests
cargo test --test storage_tests
cargo test --test vault_tests
cargo test --test integration_tests
```

**Why `--test-threads=1`?**  
The clipboard tests access the system clipboard, which can only handle one operation at a time. Running tests in parallel causes `STATUS_HEAP_CORRUPTION` errors on Windows.

---

## ⚠️ Disclaimer

This is a beta version (v0.0.1-beta). While it uses industry-standard encryption, use at your own risk. Always backup your key file.
(Ofcourse, there is no httpclient code, that would let me track you, but still, be cautious. 🔪)

---