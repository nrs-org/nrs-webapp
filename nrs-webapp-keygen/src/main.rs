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

    // print only 8 bytes, censoring the rest for security
    let prefix = 8;
    let mut key_str = vec![];
    for (i, byte) in key.iter().enumerate() {
        if i < prefix {
            key_str.push(format!("{:02x}", byte));
        } else if i == prefix {
            key_str.push("**".into());
            break;
        }
    }
    eprintln!("Generated key: {:?}", key_str);

    let base64_key = BASE64_URL_SAFE.encode(key);
    println!("{}", base64_key);
}
