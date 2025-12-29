//! Integration tests for SQL schema validation

use std::fs;

#[test]
fn test_sql_files_exist_and_readable() {
    let required_files = vec![
        "sql/dev_initial/01-prelude.sql",
        "sql/dev_initial/02-auth-create-app-user-table.sql",
        "sql/dev_initial/03-auth-create-user-token-table.sql",
        "sql/dev_initial/04-create-schema.sql",
    ];
    
    for file_path in required_files {
        let content = fs::read_to_string(file_path);
        assert!(content.is_ok(), "Should be able to read {}", file_path);
        assert!(!content.unwrap().is_empty(), "{} should not be empty", file_path);
    }
}

#[test]
fn test_user_table_has_required_fields() {
    let content = fs::read_to_string("sql/dev_initial/02-auth-create-app-user-table.sql")
        .expect("Should read user table SQL");
    
    let required_fields = vec![
        "id",
        "username",
        "email",
        "password_hash",
        "email_verified_at",
        "created_at",
        "updated_at",
    ];
    
    for field in required_fields {
        assert!(content.contains(field), "User table should have {} field", field);
    }
}

#[test]
fn test_user_table_has_constraints() {
    let content = fs::read_to_string("sql/dev_initial/02-auth-create-app-user-table.sql")
        .expect("Should read user table SQL");
    
    assert!(content.to_uppercase().contains("PRIMARY KEY"), "Should have primary key");
    assert!(content.to_uppercase().contains("UNIQUE"), "Should have unique constraints");
}

#[test]
fn test_token_table_has_required_fields() {
    let content = fs::read_to_string("sql/dev_initial/03-auth-create-user-token-table.sql")
        .expect("Should read token table SQL");
    
    let required_fields = vec![
        "user_id",
        "purpose",
        "token_hash",
        "expires_at",
        "created_at",
    ];
    
    for field in required_fields {
        assert!(content.contains(field), "Token table should have {} field", field);
    }
}

#[test]
fn test_token_table_has_foreign_key() {
    let content = fs::read_to_string("sql/dev_initial/03-auth-create-user-token-table.sql")
        .expect("Should read token table SQL");
    
    let content_lower = content.to_lowercase();
    let has_fk = content_lower.contains("foreign key") || 
                 content_lower.contains("references");
    assert!(has_fk, "Token table should have foreign key constraint");
}

#[test]
fn test_sql_files_proper_termination() {
    let files = vec![
        "sql/dev_initial/02-auth-create-app-user-table.sql",
        "sql/dev_initial/03-auth-create-user-token-table.sql",
    ];
    
    for file_path in files {
        let content = fs::read_to_string(file_path).unwrap();
        let trimmed = content.trim();
        
        if !trimmed.is_empty() {
            assert!(
                trimmed.ends_with(';'),
                "{} should end with semicolon",
                file_path
            );
        }
    }
}

#[test]
fn test_prelude_not_empty() {
    let content = fs::read_to_string("sql/dev_initial/01-prelude.sql")
        .expect("Should read prelude");
    
    assert!(!content.trim().is_empty(), "Prelude should not be empty");
}

#[test]
fn test_user_table_has_case_insensitive_collation() {
    let content = fs::read_to_string("sql/dev_initial/02-auth-create-app-user-table.sql")
        .expect("Should read user table SQL");
    
    assert!(
        content.contains("case_insensitive"),
        "Username and email should use case_insensitive collation"
    );
}

#[test]
fn test_user_table_has_timestamps() {
    let content = fs::read_to_string("sql/dev_initial/02-auth-create-app-user-table.sql")
        .expect("Should read user table SQL");
    
    assert!(content.contains("created_at"), "Should have created_at");
    assert!(content.contains("updated_at"), "Should have updated_at");
    assert!(content.to_uppercase().contains("TIMESTAMPTZ"), "Should use TIMESTAMPTZ");
}

#[test]
fn test_token_table_has_expiry() {
    let content = fs::read_to_string("sql/dev_initial/03-auth-create-user-token-table.sql")
        .expect("Should read token table SQL");
    
    assert!(content.contains("expires_at"), "Should have expires_at field");
}

#[test]
fn test_no_obvious_sql_injection_patterns() {
    let files = vec![
        "sql/dev_initial/01-prelude.sql",
        "sql/dev_initial/02-auth-create-app-user-table.sql",
        "sql/dev_initial/03-auth-create-user-token-table.sql",
        "sql/dev_initial/04-create-schema.sql",
    ];
    
    for file_path in files {
        let content = fs::read_to_string(file_path).unwrap();
        
        assert!(!content.contains("'; DROP TABLE"), "Should not contain injection pattern");
        assert!(!content.contains("' OR '1'='1"), "Should not contain injection pattern");
    }
}

#[test]
fn test_table_names_consistent() {
    let user_table_content = fs::read_to_string("sql/dev_initial/02-auth-create-app-user-table.sql").unwrap();
    let token_table_content = fs::read_to_string("sql/dev_initial/03-auth-create-user-token-table.sql").unwrap();
    
    assert!(user_table_content.contains("app_user"), "Should reference app_user table");
    assert!(token_table_content.contains("user_one_time_token"), "Should reference token table");
}