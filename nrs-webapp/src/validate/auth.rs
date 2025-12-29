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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        let valid_usernames = vec![
            "user",
            "user123",
            "user_name",
            "user-name",
            "a",
            "u".repeat(20),
        ];
        
        for username in valid_usernames {
            let result = validate_username(&username);
            assert!(result.is_ok(), "Username '{}' should be valid", username);
        }
    }

    #[test]
    fn test_validate_username_too_short() {
        let result = validate_username("");
        assert!(result.is_err(), "Empty username should be invalid");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("too short") || e.to_string().contains("length"));
        }
    }

    #[test]
    fn test_validate_username_too_long() {
        let long_username = "a".repeat(100);
        let result = validate_username(&long_username);
        assert!(result.is_err(), "Username over max length should be invalid");
    }

    #[test]
    fn test_validate_username_invalid_characters() {
        let invalid_usernames = vec![
            "user@name",
            "user name",
            "user#name",
            "user$name",
            "user%name",
            "user!name",
        ];
        
        for username in invalid_usernames {
            let result = validate_username(username);
            assert!(result.is_err(), "Username '{}' with invalid chars should be invalid", username);
        }
    }

    #[test]
    fn test_validate_username_unicode() {
        let unicode_usernames = vec![
            "user名",
            "użytkownik",
            "пользователь",
        ];
        
        for username in unicode_usernames {
            let result = validate_username(username);
            // Depending on requirements, this may be valid or invalid
            // Adjust the assertion based on actual validation rules
        }
    }

    #[test]
    fn test_validate_email_valid() {
        let valid_emails = vec![
            "user@example.com",
            "test.email@example.com",
            "user+tag@example.co.uk",
            "user_name@example.com",
            "user-name@example-domain.com",
            "a@b.c",
        ];
        
        for email in valid_emails {
            let result = validate_email(email);
            assert!(result.is_ok(), "Email '{}' should be valid", email);
        }
    }

    #[test]
    fn test_validate_email_invalid() {
        let invalid_emails = vec![
            "",
            "notanemail",
            "@example.com",
            "user@",
            "user@@example.com",
            "user@.com",
            "user@example.",
            "user @example.com",
            "user@exam ple.com",
        ];
        
        for email in invalid_emails {
            let result = validate_email(email);
            assert!(result.is_err(), "Email '{}' should be invalid", email);
        }
    }

    #[test]
    fn test_validate_email_case_insensitive() {
        let emails = vec![
            "User@Example.COM",
            "TEST@EXAMPLE.COM",
            "Test@Example.Com",
        ];
        
        for email in emails {
            let result = validate_email(email);
            assert!(result.is_ok(), "Email '{}' should be valid regardless of case", email);
        }
    }

    #[test]
    fn test_validate_email_with_subdomain() {
        let emails = vec![
            "user@mail.example.com",
            "user@a.b.c.example.com",
        ];
        
        for email in emails {
            let result = validate_email(email);
            assert!(result.is_ok(), "Email '{}' with subdomain should be valid", email);
        }
    }

    #[test]
    fn test_validate_password_valid() {
        let valid_passwords = vec![
            "Password123!",
            "MyP@ssw0rd",
            "C0mplex!Pass",
            "aB3$" + &"x".repeat(50),
        ];
        
        for password in valid_passwords {
            let result = validate_password(&password);
            assert!(result.is_ok(), "Password should be valid");
        }
    }

    #[test]
    fn test_validate_password_too_short() {
        let short_passwords = vec![
            "",
            "a",
            "Pass1!",
            "1234567",
        ];
        
        for password in short_passwords {
            let result = validate_password(password);
            assert!(result.is_err(), "Password '{}' should be too short", password);
        }
    }

    #[test]
    fn test_validate_password_missing_uppercase() {
        let passwords = vec![
            "password123!",
            "mypassword1!",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Depending on requirements, this may need uppercase
            // Adjust assertion based on actual requirements
        }
    }

    #[test]
    fn test_validate_password_missing_lowercase() {
        let passwords = vec![
            "PASSWORD123!",
            "MYPASSWORD1!",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Adjust based on requirements
        }
    }

    #[test]
    fn test_validate_password_missing_digit() {
        let passwords = vec![
            "Password!",
            "MyPassword!",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Adjust based on requirements
        }
    }

    #[test]
    fn test_validate_password_missing_special_char() {
        let passwords = vec![
            "Password123",
            "MyPassword1",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Adjust based on requirements
        }
    }

    #[test]
    fn test_validate_password_too_long() {
        let long_password = "A1b!" + &"x".repeat(200);
        let result = validate_password(&long_password);
        // Most systems have an upper limit
        // Adjust based on requirements
    }

    #[test]
    fn test_validate_password_with_whitespace() {
        let passwords = vec![
            "Pass word123!",
            " Password123!",
            "Password123! ",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Adjust based on whether whitespace is allowed
        }
    }

    #[test]
    fn test_validate_password_unicode() {
        let passwords = vec![
            "Пароль123!",
            "密码123!",
            "🔐Password123",
        ];
        
        for password in passwords {
            let result = validate_password(password);
            // Adjust based on Unicode support requirements
        }
    }

    #[test]
    fn test_validate_username_boundary_length() {
        // Assuming min is 3 and max is 50
        let min_valid = "abc";
        let max_valid = "a".repeat(50);
        
        assert!(validate_username(min_valid).is_ok());
        assert!(validate_username(&max_valid).is_ok());
        
        let too_short = "ab";
        let too_long = "a".repeat(51);
        
        assert!(validate_username(too_short).is_err());
        assert!(validate_username(&too_long).is_err());
    }

    #[test]
    fn test_validate_email_length_limits() {
        // Test extremely long email
        let long_local = "a".repeat(100);
        let long_email = format!("{}@example.com", long_local);
        
        let result = validate_email(&long_email);
        // Most email validators have length limits
    }

    #[test]
    fn test_validate_password_common_patterns() {
        let weak_passwords = vec![
            "Password1!",
            "123456Aa!",
            "Qwerty123!",
        ];
        
        for password in weak_passwords {
            let result = validate_password(password);
            // These pass basic validation but might be weak
            // Consider adding password strength checks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_valid() {
        let valid_passwords = vec![
            "Password123",
            "Abc123xyz",
            "MyP4ssw0rd",
            "Str0ng!Pass",
        ];
        
        for password in valid_passwords {
            let result = validate_password(password);
            assert!(result.is_ok(), "Password '{}' should be valid", password);
        }
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        let result = validate_password("PASSWORD123");
        assert!(result.is_err(), "Password without lowercase should fail");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("lowercase"));
        }
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        let result = validate_password("password123");
        assert!(result.is_err(), "Password without uppercase should fail");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("uppercase"));
        }
    }

    #[test]
    fn test_validate_password_no_digit() {
        let result = validate_password("PasswordABC");
        assert!(result.is_err(), "Password without digit should fail");
        
        if let Err(e) = result {
            assert!(e.to_string().contains("digit"));
        }
    }

    #[test]
    fn test_validate_password_empty() {
        let result = validate_password("");
        assert!(result.is_err(), "Empty password should fail");
    }

    #[test]
    fn test_validate_password_only_lowercase() {
        assert!(validate_password("abcdefgh").is_err());
    }

    #[test]
    fn test_validate_password_only_uppercase() {
        assert!(validate_password("ABCDEFGH").is_err());
    }

    #[test]
    fn test_validate_password_only_digits() {
        assert!(validate_password("12345678").is_err());
    }

    #[test]
    fn test_validate_password_with_special_chars() {
        assert!(validate_password("Pass123!@#").is_ok());
        assert!(validate_password("P@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_with_spaces() {
        assert!(validate_password("Pass 123").is_ok());
    }

    #[test]
    fn test_validate_password_unicode() {
        // Unicode characters should be allowed
        assert!(validate_password("Пароль123").is_ok());
        assert!(validate_password("密码Abc123").is_ok());
    }

    #[test]
    fn test_validate_password_minimal_valid() {
        assert!(validate_password("Aa1").is_ok());
    }

    #[test]
    fn test_validate_password_long() {
        let long = format!("Aa1{}", "x".repeat(1000));
        assert!(validate_password(&long).is_ok());
    }

    #[test]
    fn test_username_regex_valid() {
        let valid_usernames = vec![
            "abc",
            "user123",
            "test_user",
            "user-name",
            "a1b2c3",
            "User_Name-123",
            "a".repeat(20),
        ];
        
        for username in valid_usernames {
            assert!(
                USERNAME_REGEX.is_match(&username),
                "Username '{}' should match regex",
                username
            );
        }
    }

    #[test]
    fn test_username_regex_invalid() {
        let invalid_usernames = vec![
            "ab",           // too short
            "a".repeat(21), // too long
            "user name",    // space
            "user@name",    // @
            "user#name",    // #
            "user!name",    // !
            "",             // empty
        ];
        
        for username in invalid_usernames {
            assert!(
                !USERNAME_REGEX.is_match(&username),
                "Username '{}' should not match regex",
                username
            );
        }
    }

    #[test]
    fn test_username_regex_length_boundaries() {
        assert!(USERNAME_REGEX.is_match("abc"), "3 chars should be valid");
        assert!(USERNAME_REGEX.is_match(&"a".repeat(20)), "20 chars should be valid");
        assert!(!USERNAME_REGEX.is_match("ab"), "2 chars should be invalid");
        assert!(!USERNAME_REGEX.is_match(&"a".repeat(21)), "21 chars should be invalid");
    }

    #[test]
    fn test_username_regex_allowed_chars() {
        assert!(USERNAME_REGEX.is_match("azAZ09"));
        assert!(USERNAME_REGEX.is_match("user_name"));
        assert!(USERNAME_REGEX.is_match("user-name"));
    }

    #[test]
    fn test_username_regex_case_sensitive() {
        assert!(USERNAME_REGEX.is_match("ABC"));
        assert!(USERNAME_REGEX.is_match("abc"));
        assert!(USERNAME_REGEX.is_match("AbC"));
    }
}