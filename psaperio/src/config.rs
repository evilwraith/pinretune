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
    bytes::complete::{take_till1, take_until, take_while},
    character::complete::{alphanumeric1, digit1, hex_digit1, space1},
    sequence::terminated,
    IResult,
};

use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use xsalsa20poly1305::{AeadInPlace, KeyInit, XSalsa20Poly1305};

/// Pinsound Configuration Primitives
pub const APPVERSION: &str = "versionDetail";
pub const CHECKSUM: &str = "checksum";
pub const CHECKSUMS: &str = "checksums";
pub const CHECKSUMVER: &str = "checksum_version";
pub const CONCURRENT: &str = "max";
pub const CONFIGVER: &str = "versionConfigFile";
pub const DATETIME: &str = "timestamp";
pub const DUCKING: &str = "ducking";
pub const GAIN: &str = "gain";
pub const PACKID: &str = "pspack_id";
pub const RECORD: &str = "record_enabled";

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
    APPVERSION(String),
    CHECKSUM(String),
    CHECKSUMS(Vec<String>),
    CHECKSUMVER(u32),
    CONCURRENT(u32),
    CONFIGVER(u32),
    DATETIME(u32),
    DUCKING,
    FALLBACK(String),
    GAIN,
    PACKID(u64),
    RECORD,
    SOUND,
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::APPVERSION(v) => write!(f, "{} {}", APPVERSION, v),
            Primitive::CHECKSUM(v) => write!(f, "{} {}", CHECKSUM, v),
            Primitive::CHECKSUMS(v) => write!(f, "{} {}", CHECKSUMS, v.join(" ")),
            Primitive::CHECKSUMVER(v) => write!(f, "{} {}", CHECKSUMVER, v),
            Primitive::CONCURRENT(v) => write!(f, "{} {}", CONCURRENT, v),
            Primitive::CONFIGVER(v) => write!(f, "{} {}", CONFIGVER, v),
            Primitive::DATETIME(v) => write!(f, "{} {}", DATETIME, v),
            Primitive::DUCKING => write!(f, "{}", DUCKING),
            Primitive::FALLBACK(v) => write!(f, "{}", v),
            Primitive::GAIN => write!(f, "{}", GAIN),
            Primitive::PACKID(v) => write!(f, "{} {:0x}", PACKID, v),
            Primitive::RECORD => write!(f, "{}", RECORD),
            Primitive::SOUND => write!(f, "xxx"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Action {
    DELAY,
    GAIN,
    SHAKER,
    STOP,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::DELAY => write!(f, "delay"),
            Action::GAIN => write!(f, "{}", GAIN),
            Action::SHAKER => write!(f, "shakerPreset"),
            Action::STOP => write!(f, "stop"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
    JINGLE,
    MUSIC,
    SFX,
    SINGLE,
    VOICE,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::JINGLE => write!(f, "jingle"),
            Kind::MUSIC => write!(f, "music"),
            Kind::SFX => write!(f, "sfx"),
            Kind::SINGLE => write!(f, "single"),
            Kind::VOICE => write!(f, "voice"),
        }
    }
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

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    terminated(take_till1(char::is_whitespace), space1)(input)
}

fn parse_statement(input: &str) -> IResult<&str, &str> {
    terminated(take_until("|"), take_while(|c| c == '|' || c == ' '))(input)
}

fn parse_config(cfg: &mut PsConfig, input: &str) -> PsConfig {
    let mut data = input;
    let mut config = PsConfig::new();

    while let Ok((rest, val)) = parse_statement(data) {
        match parse_identifier(val) {
            Ok((rest, val)) => {
                match val {
                    APPVERSION => {
                        if let Ok((_, v)) = parse_alnum(rest) {
                            config.0.insert(APPVERSION.to_string(), v.to_string());
                        }
                    }
                    CHECKSUM => {
                        if let Ok((_, v)) = parse_alnum(rest) {
                            println!("Insert {} = {}", val, v);
                            config.0.insert(CHECKSUM.to_string(), v.to_string());
                        }
                    }
                    CHECKSUMVER => {
                        if let Ok((_, v)) = parse_hex(rest) {
                            config.0.insert(CHECKSUMS.to_string(), v.to_string());
                        }
                    }
                    CONCURRENT => {
                        if let Ok((_, v)) = parse_alnum(rest) {
                            config.0.insert(CONCURRENT.to_string(), v.to_string());
                        }
                    }
                    CONFIGVER => {
                        if let Ok((_, v)) = parse_hex(rest) {
                            config.0.insert(CONFIGVER.to_string(), v.to_string());
                        }
                    }
                    DATETIME => {
                        if let Ok((_, v)) = parse_number(rest) {
                            config.0.insert(DATETIME.to_string(), v.to_string());
                        }
                    }
                    DUCKING => {
                        if let Ok((_, v)) = parse_alnum(rest) {
                            config.0.insert(DUCKING.to_string(), v.to_string());
                        }
                    }
                    PACKID => {
                        if let Ok((_, v)) = parse_hex(rest) {
                            config.0.insert(PACKID.to_string(), v.to_string());
                        }
                    }
                    RECORD => {
                        if let Ok((_, v)) = parse_hex(rest) {
                            config.0.insert(RECORD.to_string(), v.to_string());
                        }
                    }
                    &_ => println!("{} not inplemented", val),
                };
            }
            Err(_) => break,
        }
        data = rest;
    }
    config
}

#[derive(Debug, Default)]
/// Struct representing a PS configuration file.
///
/// *The configuration file format is not exclusive to soundpacks.*
pub struct PsConfig(HashMap<String, String>);

impl PsConfig {
    /// Create a new empty PsConfig struct
    pub fn new() -> Self {
        PsConfig(HashMap::new())
    }

    /// Initialize struct by decrypting and parsing a PS config file.
    /// This is a destructive action and will clear the current struct state.
    ///
    /// # Arguments
    /// * `filename` - The name of the file to read from
    pub fn load<F: AsRef<Path>>(&mut self, filename: F) -> Result<PsConfig, std::io::Error> {
        let mut file = File::open(filename)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;
        let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
        cipher.decrypt_in_place(PS_NONCE.into(), b"", &mut data);

        let cfg_str = String::from_utf8_lossy(&data);
        let config = parse_config(self, &cfg_str);

        Ok(config)
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

use std::ops::Deref;
impl Deref for PsConfig {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTCONF: &str = "checksum_version 1|checksum Test|";

    #[test]
    fn checksum() {
        let (l, r) = parse_statement(TESTCONF).unwrap();
        assert_eq!(r, CHECKSUMVER);
    }
}
