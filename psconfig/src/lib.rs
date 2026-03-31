// Copyright 2026 Martin Åkesson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing
// permissions and limitations under the License.

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
