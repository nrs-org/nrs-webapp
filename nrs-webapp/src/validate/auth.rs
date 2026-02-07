use regex_macro::{LazyRegex, lazy_regex};
use validator::ValidationError;

pub static USERNAME_REGEX: LazyRegex = lazy_regex!(r"^[A-Za-z0-9_\-]{3,20}$");

/// Validates that a password contains at least one ASCII lowercase letter, one ASCII uppercase letter, and one ASCII digit.
///
/// Returns `Ok(())` when the password meets all three requirements. Returns `Err(ValidationError)` with one of the messages:
/// - "Password must contain at least one lowercase letter"
/// - "Password must contain at least one uppercase letter"
/// - "Password must contain at least one digit"
///
/// # Examples
///
/// ```
/// assert!(validate_password("Abc1").is_ok());
/// assert!(validate_password("abc").is_err());
/// ```
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::new("password_at_least_one_lowercase")
            .with_message("Password must contain at least one lowercase letter".into()));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new("password_at_least_one_uppercase")
            .with_message("Password must contain at least one uppercase letter".into()));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_at_least_one_digit")
            .with_message("Password must contain at least one digit".into()));
    }

    Ok(())
}
