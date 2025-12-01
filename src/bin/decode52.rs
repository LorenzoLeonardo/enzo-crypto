use enzo_crypto::{base52::Base52Codec, util};
use std::error::Error; // assuming your crate name is enzo_crypto

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <base52 string>", args[0]);
        std::process::exit(1);
    }
    let codec = Base52Codec;

    let decoded = codec.decode(util::data_source(&args[1])?)?; // Validate input
    println!("[Decoded Text] {}", String::from_utf8(decoded)?);

    Ok(())
}
