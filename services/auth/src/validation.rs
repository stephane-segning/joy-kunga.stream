//! Input validation utilities

use regex::Regex;
use std::sync::OnceLock;

/// Validate username
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("Username is required".to_string());
    }

    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if username.len() > 32 {
        return Err("Username must be at most 32 characters long".to_string());
    }

    static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = USERNAME_REGEX
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9_]+$").expect("Failed to compile username regex"));

    if !regex.is_match(username) {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }

    Ok(())
}

/// Validate email
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email is required".to_string());
    }

    if email.len() > 254 {
        return Err("Email must be at most 254 characters long".to_string());
    }

    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Failed to compile email regex")
    });

    if !regex.is_match(email) {
        return Err("Invalid email format".to_string());
    }

    Ok(())
}

/// Validate password
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.is_empty() {
        return Err("Password is required".to_string());
    }

    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if password.len() > 128 {
        return Err("Password must be at most 128 characters long".to_string());
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        if c.is_ascii_uppercase() {
            has_upper = true;
        } else if c.is_ascii_lowercase() {
            has_lower = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if !c.is_alphanumeric() {
            has_special = true;
        }
    }

    if !has_upper {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !has_lower {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }

    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}
