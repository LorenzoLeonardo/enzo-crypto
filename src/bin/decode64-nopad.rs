use base64::{Engine, engine::general_purpose};
use std::{error::Error, fs, path::Path}; // assuming your crate name is enzo_crypto

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <base64 string>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    let data: Vec<u8> = if path.exists() && path.is_file() {
        // Definitely a real file
        fs::read(path)?
    } else {
        // Not a real file → treat as literal string
        args[1].as_bytes().to_vec()
    };

    let decoded = general_purpose::STANDARD_NO_PAD.decode(data)?;
    println!("[Decoded Text] {}", String::from_utf8(decoded)?);

    Ok(())
}
