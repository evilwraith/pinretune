//! # Configuration manager
//! Provides a simple interface to work with PS configuration files.
//!
//! ## Example
//! ``` rust
//! # use psconfig::config::PsConfig;
//! let config = PsConfig::new();
//!_ = config.load("config.pinsound");
//! ```

use nom::{
    bytes::complete::{take_till1, take_until, take_while},
    character::complete::{alphanumeric1, digit1, hex_digit1, space1},
    sequence::terminated,
    IResult,
};
use std::collections::HashMap;
use std::fmt;

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
pub const RECORDING: &str = "record_enabled";

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
            Primitive::RECORD => write!(f, "{}", RECORDING),
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
                    RECORDING => {
                        if let Ok((_, v)) = parse_hex(rest) {
                            config.0.insert(RECORDING.to_string(), v.to_string());
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

    pub fn str(data: &str) -> Result<PsConfig, std::io::Error> {
        let mut conf = PsConfig::new();
        let config = parse_config(&mut conf, data);
        Ok(config)
    }

    pub fn as_str(self, data: &PsConfig) -> Result<String, std::io::Error> {
        todo!()
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
