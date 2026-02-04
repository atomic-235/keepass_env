//! keyring-env: Fast keyring to environment variable loader
//!
//! Reads a bundle of secrets from GNOME Keyring and outputs shell export statements.
//! Designed for near-instant shell startup.
//!
//! Usage:
//!   eval "$(keyring-env)"
//!
//! The bundle is stored as a single keyring entry with:
//!   - service: keepass_env_bundle
//!   - name: bundle
//!   - value: newline-separated KEY=base64(VALUE) pairs

use base64::{engine::general_purpose::STANDARD, Engine};
use oo7::Keyring;
use std::collections::HashMap;

const SERVICE: &str = "keepass_env_bundle";
const NAME: &str = "bundle";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("keyring-env: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let keyring = Keyring::new().await?;

    // Build attributes to search for
    let mut attrs = HashMap::new();
    attrs.insert("service", SERVICE);
    attrs.insert("name", NAME);

    // Search for the item
    let items = keyring.search_items(&attrs).await?;

    let item = items.first().ok_or("No secrets bundle found. Run keepass-sync first.")?;

    // Unlock if needed and get the secret
    let secret = item.secret().await?;
    let bundle = String::from_utf8(secret.to_vec())?;

    // Parse and output export statements
    for line in bundle.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, b64val)) = line.split_once('=') {
            // Validate key is a valid shell variable name
            if !is_valid_var_name(key) {
                eprintln!("keyring-env: skipping invalid var name: {}", key);
                continue;
            }

            // Decode base64 value
            match STANDARD.decode(b64val) {
                Ok(decoded) => {
                    if let Ok(value) = String::from_utf8(decoded) {
                        // Output shell-safe export statement
                        println!("export {}={}", key, shell_quote(&value));
                    }
                }
                Err(e) => {
                    eprintln!("keyring-env: failed to decode {}: {}", key, e);
                }
            }
        }
    }

    Ok(())
}

/// Check if a string is a valid shell variable name
fn is_valid_var_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    // First char must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Quote a string for safe use in shell
fn shell_quote(s: &str) -> String {
    // Use single quotes, escaping any single quotes in the value
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}
