use simple_enc_dec::{self, scrypt};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <base64 string>", args[0]);
        std::process::exit(1);
    }

    let base64_cipher_text = &args[1];
    let password = &args[2];

    let plaintext =
        scrypt::decrypt_base64(base64_cipher_text, password).map(String::from_utf8)??;
    println!("[Decrypted Text] {plaintext}");

    Ok(())
}
