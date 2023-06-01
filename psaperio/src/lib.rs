/*!
PS Aperio - A PinSound Utility Library
======================================

This library provides utility functions for PinSound[^PinSound] sound packs and configuration
files.

Currently, this library provides the following:
- [x] Scrambling and unscrambling of sound command IDs.
- [x] Reading encrypted configuration files
- [x] Partial parsing configuration files
- [ ] Modifying configuration files
- [ ] Writing encrypted configuration files

[^PinSound]: *This library is not affiliated, associated, authorized, endorsed by, or in any way
connected with PinSound, SARL, or any of its subsidiaries or its affiliates.
PinSound is a registered trademark of PinSound, SARL.*
*/

mod bytes;
pub mod config;
pub mod scrambler;
pub mod util;
