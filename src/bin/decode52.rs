use enzo_crypto::base52::Base52Codec;
use std::{error::Error, fs, path::Path}; // assuming your crate name is enzo_crypto

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <base52 string>", args[0]);
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

    let codec = Base52Codec;

    let decoded = codec.decode(data)?; // Validate input
    println!(
        "[Decoded Text] {}",
        decoded.iter().map(|&b| b as char).collect::<String>()
    );

    Ok(())
}
