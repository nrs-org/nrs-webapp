use base64::{Engine, prelude::BASE64_URL_SAFE};

/// Generates a 128-byte random key, prints the raw key (debug format) to stderr, and writes a URL-safe Base64 encoding of the key to stdout.
///
/// # Examples
///
/// ```
/// // Run the program's main entry point; this prints a Base64 key to stdout.
/// main();
/// ```
fn main() {
    const KEY_LENGTH: usize = 128;

    let key: [u8; KEY_LENGTH] = rand::random();
    eprintln!("Generated key: {:?}", key);

    let base64_key = BASE64_URL_SAFE.encode(&key);
    println!("{}", base64_key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_length() {
        let key = generate_key();
        
        // 64 random bytes encoded as base64 should be around 88 characters
        assert!(key.len() >= 85 && key.len() <= 90, "Key length should be appropriate for base64-encoded 64 bytes");
    }

    #[test]
    fn test_generate_key_uniqueness() {
        let key1 = generate_key();
        let key2 = generate_key();
        
        assert_ne!(key1, key2, "Generated keys should be unique");
    }

    #[test]
    fn test_generate_key_not_empty() {
        let key = generate_key();
        
        assert!(!key.is_empty(), "Generated key should not be empty");
    }

    #[test]
    fn test_generate_key_valid_base64() {
        let key = generate_key();
        
        // Verify it's valid base64
        let decode_result = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &key
        );
        
        assert!(decode_result.is_ok(), "Generated key should be valid base64");
        
        let decoded = decode_result.unwrap();
        assert_eq!(decoded.len(), 64, "Decoded key should be 64 bytes");
    }

    #[test]
    fn test_generate_key_multiple_calls() {
        let mut keys = std::collections::HashSet::new();
        
        for _ in 0..100 {
            let key = generate_key();
            assert!(!keys.contains(&key), "Each generated key should be unique");
            keys.insert(key);
        }
        
        assert_eq!(keys.len(), 100, "Should generate 100 unique keys");
    }

    #[test]
    fn test_generate_key_character_set() {
        let key = generate_key();
        
        // Base64 should only contain alphanumeric, +, /, and =
        for ch in key.chars() {
            assert!(
                ch.is_alphanumeric() || ch == '+' || ch == '/' || ch == '=',
                "Key should only contain valid base64 characters"
            );
        }
    }

    #[test]
    fn test_generate_key_entropy() {
        let key = generate_key();
        
        // Basic entropy check: ensure it's not all the same character
        let first_char = key.chars().next().unwrap();
        let all_same = key.chars().all(|c| c == first_char);
        
        assert!(!all_same, "Key should have varied characters (entropy check)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_length_constant() {
        const KEY_LENGTH: usize = 128;
        assert_eq!(KEY_LENGTH, 128, "Key length should be 128 bytes");
    }

    #[test]
    fn test_random_key_generation() {
        const KEY_LENGTH: usize = 128;
        let key1: [u8; KEY_LENGTH] = rand::random();
        let key2: [u8; KEY_LENGTH] = rand::random();
        
        assert_ne!(key1, key2, "Random keys should be different");
    }

    #[test]
    fn test_base64_encoding_length() {
        const KEY_LENGTH: usize = 128;
        let key: [u8; KEY_LENGTH] = rand::random();
        let encoded = BASE64_URL_SAFE.encode(&key);
        
        // 128 bytes -> ~171 base64 characters
        assert!(encoded.len() >= 170 && encoded.len() <= 175, 
                "Base64 of 128 bytes should be ~171 chars, got {}", encoded.len());
    }

    #[test]
    fn test_base64_roundtrip() {
        const KEY_LENGTH: usize = 128;
        let original: [u8; KEY_LENGTH] = rand::random();
        let encoded = BASE64_URL_SAFE.encode(&original);
        let decoded = BASE64_URL_SAFE.decode(&encoded).unwrap();
        
        assert_eq!(original.to_vec(), decoded, "Roundtrip should preserve data");
    }

    #[test]
    fn test_base64_url_safe_characters() {
        const KEY_LENGTH: usize = 128;
        let key: [u8; KEY_LENGTH] = rand::random();
        let encoded = BASE64_URL_SAFE.encode(&key);
        
        for ch in encoded.chars() {
            assert!(
                ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '=',
                "URL-safe base64 should only contain alphanumeric, -, _, or ="
            );
        }
    }

    #[test]
    fn test_multiple_keys_unique() {
        const KEY_LENGTH: usize = 128;
        let mut keys = std::collections::HashSet::new();
        
        for _ in 0..10 {
            let key: [u8; KEY_LENGTH] = rand::random();
            let encoded = BASE64_URL_SAFE.encode(&key);
            assert!(!keys.contains(&encoded), "Each key should be unique");
            keys.insert(encoded);
        }
        
        assert_eq!(keys.len(), 10, "Should have 10 unique keys");
    }

    #[test]
    fn test_key_entropy() {
        const KEY_LENGTH: usize = 128;
        let key: [u8; KEY_LENGTH] = rand::random();
        
        // Check that not all bytes are the same
        let first = key[0];
        let all_same = key.iter().all(|&b| b == first);
        
        assert!(!all_same, "Key should have entropy (not all bytes the same)");
    }
}