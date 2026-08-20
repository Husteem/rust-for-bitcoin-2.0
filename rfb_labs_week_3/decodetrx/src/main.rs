use clap::{Arg, Command};
use decodetrx::decode_transaction;

fn main() {
    let matches = Command::new("decodetrx")
        .version("0.1.0")
        .about("Decodes a raw Bitcoin transaction hex into JSON format")
        .arg(
            Arg::new("hex")
                .help("The raw transaction hex string")
                .required(true)
                .index(1),
        )
        .get_matches();

    let hex_str = matches.get_one::<String>("hex").unwrap();

    match decode_transaction(hex_str.to_string()) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error decoding transaction: {}", e);
            std::process::exit(1);
        }
    }
}
