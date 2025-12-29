# Test Suite Documentation

This directory contains integration and unit tests for the nrs-webapp authentication system.

## Test Organization

### Unit Tests
Unit tests are located within each module using `#[cfg(test)]`:
- `src/crypt/password_hash.rs` - Password hashing (Argon2) tests
- `src/crypt/jwt.rs` - JWT token generation and validation
- `src/crypt/token.rs` - Secure token generation (HMAC-SHA256)
- `src/validate/auth.rs` - Input validation (username, email, password)
- `src/toasts/mod.rs` - Toast notification system
- `nrs-webapp-keygen/src/main.rs` - Cryptographic key generation

### Integration Tests
- `tests/sql_schema_validation.rs` - SQL schema structure and consistency

## Running Tests

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test --package nrs-webapp
cargo test --package nrs-webapp-keygen
cargo test --package nrs-webapp-frontend

# Run with output
cargo test -- --nocapture --test-threads=1

# Run integration tests only
cargo test --test sql_schema_validation

# Run a specific test
cargo test test_token_generate_success
```

## Test Coverage

### Authentication & Security
- Password hashing with Argon2 (salting, pepper, verification)
- JWT token creation, signing, and validation
- Secure token generation for email verification and password reset
- Token HMAC hashing and verification
- Input validation with regex patterns

### Database Schema
- Table structure validation
- Foreign key constraints
- Unique constraints
- Index definitions
- Timestamp fields

### Utilities
- Cryptographic key generation
- Base64 encoding/decoding
- Toast notification system

## Best Practices

1. **Test Isolation**: Each test is independent
2. **Descriptive Names**: `test_<module>_<scenario>`
3. **Comprehensive Coverage**: Happy paths + edge cases + error conditions
4. **Fast Execution**: Use mocks for external dependencies
5. **Deterministic Results**: No flaky tests

## Adding New Tests

When adding tests:
1. Place unit tests in the same file as the code
2. Use descriptive names: `test_function_name_scenario`
3. Test at minimum:
   - Success case
   - Common failures
   - Edge cases (empty, long input, unicode)
   - Boundary values

## Test Examples

### Good Test Structure
```rust
#[test]
fn test_validate_password_with_all_requirements() {
    let password = "SecurePass123";
    let result = validate_password(password);
    assert!(result.is_ok());
}

#[test]
fn test_validate_password_missing_digit() {
    let password = "SecurePassword";
    let result = validate_password(password);
    assert!(result.is_err());
}
```

## Continuous Integration

Tests run automatically on:
- Pull requests
- Commits to main
- Release builds

All tests must pass before merging.