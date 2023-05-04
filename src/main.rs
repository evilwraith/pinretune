use psaperio::{config, scrambler::IdScrambler};
use std::num::ParseIntError;

fn convert_id_str(checksum: &str, id: &str) -> Result<String, ParseIntError> {
    // TODO: add error handling
    let scrambler = IdScrambler::new(checksum);
    let val = id.parse::<u32>()?;
    let result = match id.len() {
        1..=4 => {
            format!("{:010}", scrambler.scramble(val))
        }
        5..=10 => {
            format!("{:04}", scrambler.unscramble(val))
        }
        _ => panic!("Invalid ID length"),
    };
    Ok(result)
}

fn main() {
    let config = config::PsConfig::new();
    let apa = config.load("config.pinsound");

    // println!("{}", convert_id_str("JeanLucPicard", "0010").unwrap());
    // println!("{}", convert_id_str("JeanLucPicard", "2824050793").unwrap());
}
