//! Parser and data model for PinSound configuration files.
//!
//! A PinSound configuration file is a flat sequence of pipe-delimited
//! statements, each beginning with a keyword or numeric sound ID followed
//! by optional arguments. This crate provides:
//!
//! - [`config::PsConfig`] — the parsed configuration model with typed accessors.
//! - [`config::Statement`] — individual typed statement variants.
//! - [`parse::parse`] — parse a configuration string into a [`config::PsConfig`].
//! - [`error::ParseError`] — error type returned by the parser.
//!
//! Unknown statements are preserved verbatim so files can be round-tripped
//! without data loss.
//!
//! ## Example
//! ```rust
//! use psconfig::parse;
//!
//! let input = "checksum JeanLucPicard|max sfx 3|";
//! let config = parse::parse(input).unwrap();
//! assert_eq!(config.checksum(), Some("JeanLucPicard"));
//! ```

pub mod config;
pub mod error;
pub mod parse;
