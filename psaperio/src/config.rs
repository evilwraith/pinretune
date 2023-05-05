//! # Configuration manager
//! Provides a simple interface to work with PS configuration files.
//!
//! ## Example
//! ``` rust
//! # use psaperio::config::PsConfig;
//! let config = PsConfig::new();
//!_ = config.load("config.pinsound");
//! ```

use nom::{
    bytes::complete::{tag, take_till1, take_until},
    character::complete::{alphanumeric1, digit1, hex_digit1, space1},
    combinator::recognize,
    multi::separated_list1,
    sequence::terminated,
    IResult,
};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
};
use xsalsa20poly1305::{AeadInPlace, KeyInit, XSalsa20Poly1305};

// PinSound file encryption key (256bit)
// Note: Key is not stored in original PS binaries but generated on demand
static PS_KEY: &[u8; 32] = include_bytes!("extra/key.bin");

// PinSound file encryption nonce (192bit)
static PS_NONCE: &[u8; 24] = &[
    0x4e, 0x3e, 0x68, 0x72, 0x10, 0xa0, 0xa3, 0x27, 0x5a, 0x00, 0xd2, 0x5b, 0x68, 0xd3, 0xf8, 0xba,
    0x00, 0x43, 0x33, 0x95, 0x90, 0x9e, 0xdb, 0xf7,
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Primitive {
    APPVER(String),
    CHECKSUM(String),
    CHECKSUMS(Vec<String>),
    CHECKSUMVER(u32),
    CONFIGVER(u32),
    DATETIME(u32),
    DUCKING(Kind),
    FALLBACK(String),
    GAIN(Kind),
    MAX(Kind),
    RECORD(bool),
    SOUND(Action),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    DELAY(u32),
    GAIN(u32),
    SHAKER(u32),
    STOP(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
    JINGLE(u32),
    MUSIC(u32),
    SFX(u32),
    SINGLE(u32),
    VOICE(u32),
}

fn parse_hex(input: &str) -> IResult<&str, &str> {
    hex_digit1(input)
}

fn parse_alnum(input: &str) -> IResult<&str, &str> {
    alphanumeric1(input)
}

fn parse_number(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

fn parse_sound(input: &str) -> IResult<&str, &str> {
    let (rest, val) = parse_identifier(input).unwrap();
    parse_number(rest)
}

fn parse_fallback(input: &str) -> IResult<&str, Primitive> {
    match recognize(take_until("|"))(input) {
        Ok((rest, val)) => Ok((rest, Primitive::FALLBACK(val.to_string()))),
        Err(e) => Err(e),
    }
}

fn parse_checksum(input: &str) -> IResult<&str, Primitive> {
    match alphanumeric1(input) {
        Ok((rest, val)) => Ok((rest, Primitive::CHECKSUM(val.to_string()))),
        Err(e) => Err(e),
    }
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    terminated(take_till1(char::is_whitespace), space1)(input)
}

fn parse_statement(input: &str) -> IResult<&str, Primitive> {
    match parse_identifier(input) {
        Ok((rest, val)) => {
            let f = match val {
                "checksum" => parse_checksum,
                // "checksum_version" => parse_hex,
                // "ducking" => parse_sound,
                // "gain" => parse_sound,
                // "max" => parse_sound,
                _ => parse_fallback,
            };
            f(rest)
        }
        Err(e) => Err(e),
    }
}

fn parse_config(input: &str) -> IResult<&str, Vec<Primitive>> {
    separated_list1(tag("|"), parse_statement)(input)
}

/// Struct representing a PS configuration file.
///
/// *The configuration file format is not exclusive to soundpacks.*
pub struct PsConfig {
    elements: HashSet<String>,
    version: u32,
}

impl PsConfig {
    /// Create a new empty PsConfig struct
    pub fn new() -> Self {
        PsConfig {
            elements: HashSet::new(),
            version: 1,
        }
    }

    pub fn version(self) -> u32 {
        self.version
    }
    /// Initialize struct by decrypting and parsing a PS config file.
    /// This is a destructive action and will clear the current struct state.
    ///
    /// # Arguments
    /// * `filename` - The name of the file to read from
    pub fn load(mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(filename)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;
        let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
        cipher.decrypt_in_place(PS_NONCE.into(), b"", &mut data);

        let cfg_str = String::from_utf8_lossy(&data);
        println!("{}", cfg_str);

        let (cfg_str, ver) = match parse_config(&cfg_str) {
            Err(error) => panic!("shit"),
            Ok(result) => result,
        };
        println!("a-> '{:?}'\nb-> '{}'", ver, cfg_str);

        Ok(())
    }

    /// Write the current struct state to a encrypted PS configuration file.
    ///
    /// # Arguments
    /// * `filename` - The name of the file to write to
    pub fn save(self, filename: &str, mut data: Vec<u8>) -> Result<(), std::io::Error> {
        let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
        _ = cipher.encrypt_in_place(PS_NONCE.into(), b"", &mut data);
        let mut file = File::create(filename)?;
        file.write_all(&data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum() {
        let t = parse_statement("checksum Test").unwrap();
        assert_eq!(t, ("", Primitive::CHECKSUM("Test".to_string())));
    }
}
