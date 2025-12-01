use base64::{Engine, engine::general_purpose};
use enzo_crypto::util;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <base64 string>", args[0]);
        std::process::exit(1);
    }
    let decoded = general_purpose::STANDARD_NO_PAD.decode(util::data_source(&args[1])?)?;
    println!("[Decoded Text] {}", String::from_utf8(decoded)?);

    Ok(())
}
