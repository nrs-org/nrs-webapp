use regex_macro::{LazyRegex, lazy_regex};
use validator::ValidationError;

pub static USERNAME_REGEX: LazyRegex = lazy_regex!(r"^[A-Za-z0-9_\-]{3,20}$");

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::new(
            "Password must contain at least one lowercase letter",
        ));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new(
            "Password must contain at least one uppercase letter",
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(
            "Password must contain at least one digit",
        ));
    }

    Ok(())
}
