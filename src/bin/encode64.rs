use base64::{Engine, engine::general_purpose};
use enzo_crypto::util;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <plaintext>", args[0]);
        std::process::exit(1);
    };
    println!(
        "[Encoded Text] {}",
        general_purpose::STANDARD.encode(util::data_source(&args[1])?)
    );

    Ok(())
}
