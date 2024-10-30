// src/cli.rs
use anyhow::Result;
use rpassword::read_password;
use std::io::{self, Write};
use tracing::{info, warn, error};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    EnvFilter,
};
use zeroize::Zeroizing;


pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Logging initialized");
}

pub fn get_password_from_prompt(prompt: &str) -> Result<Zeroizing<String>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let password = match read_password() {
        Ok(pass) => Zeroizing::new(pass),
        Err(e) => {
            error!("Failed to read password: {}", e);
            return Err(anyhow::anyhow!("Failed to read password: {}", e));
        }
    };

    if password.is_empty() {
        warn!("Empty password provided");
        return Err(anyhow::anyhow!("Password cannot be empty"));
    }

    Ok(password)
}

pub fn get_confirmed_password(prompt: &str) -> Result<Zeroizing<String>> {
    let password = get_password_from_prompt(prompt)?;
    let confirmation = get_password_from_prompt("Confirm password: ")?;

    if password != confirmation {
        error!("Password confirmation failed");
        return Err(anyhow::anyhow!("Passwords do not match"));
    }

    Ok(password)
}

pub fn get_master_key() -> Result<Zeroizing<String>> {
    const MIN_LENGTH: usize = 12;

    let master_key = get_password_from_prompt("Enter master key: ")?;

    if master_key.len() < MIN_LENGTH {
        error!("Master key too short");
        return Err(anyhow::anyhow!("Master key must be at least {} characters long", MIN_LENGTH));
    }

    let has_upper = master_key.chars().any(|c| c.is_uppercase());
    let has_lower = master_key.chars().any(|c| c.is_lowercase());
    let has_digit = master_key.chars().any(|c| c.is_digit(10));
    let has_special = master_key.chars().any(|c| !c.is_alphanumeric());

    if !(has_upper && has_lower && has_digit && has_special) {
        error!("Master key does not meet complexity requirements");
        return Err(anyhow::anyhow!(
            "Master key must contain uppercase, lowercase, numbers, and special characters"
        ));
    }

    Ok(master_key)
}

pub fn print_warning(msg: &str) {
    println!("\x1b[33mWarning: {}\x1b[0m", msg);
}

pub fn print_success(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn print_error(msg: &str) {
    eprintln!("\x1b[31mError: {}\x1b[0m", msg);
}

pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("same", "same"));
        assert!(!secure_compare("different", "strings"));
        assert!(!secure_compare("short", "longer"));
    }

    #[test]
    fn test_password_strength_validation() {
        let weak_passwords = vec![
            "short",
            "nouppercase123!",
            "NOLOWERCASE123!",
            "NoSpecialChars123",
            "No!Numbers!Here!",
        ];

        for password in weak_passwords {
            let result = validate_password_strength(password);
            assert!(result.is_err(), "Password should be rejected: {}", password);
        }
    }

    fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 12 {
            return Err(anyhow::anyhow!("Password too short"));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !(has_upper && has_lower && has_digit && has_special) {
            return Err(anyhow::anyhow!("Password does not meet complexity requirements"));
        }

        Ok(())
    }
}
