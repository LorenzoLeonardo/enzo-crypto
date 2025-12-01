use enzo_crypto::{base52::Base52Codec, util};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <plaintext>", args[0]);
        std::process::exit(1);
    }
    let codec = Base52Codec;

    println!(
        "[Encoded Text] {}",
        codec.encode(util::data_source(&args[1])?)
    );

    Ok(())
}
