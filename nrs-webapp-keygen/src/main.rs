use base64::{Engine, prelude::BASE64_URL_SAFE};

fn main() {
    const KEY_LENGTH: usize = 128;

    let key: [u8; KEY_LENGTH] = rand::random();
    eprintln!("Generated key: {:?}", key);

    let base64_key = BASE64_URL_SAFE.encode(&key);
    println!("{}", base64_key);
}
