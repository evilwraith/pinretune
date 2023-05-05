//! PS Aperio - A PinSound Utility Library
//! ======================================
//!
//! This library provides utility functions for PinSound sound packs and configuration
//! files.
//!
//! Currently, this library provides the following:
//! * Reading and writing of encrypted configuration files.
//! * Scrambling and unscrambling of sound IDs.
//! * Assorted utility functions for working with sound packs.
//!
//! *This library is not affiliated, associated, authorized, endorsed by, or in any way
//! connected with PinSound, SARL, or any of its subsidiaries or its affiliates.*
//!
//! *PinSound is a registered trademark of PinSound, SARL.*

mod bytes;
pub mod config;
pub mod scrambler;
pub mod util;
