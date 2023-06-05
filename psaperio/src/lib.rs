/*!
PS Aperio - A PinSound Utility Library
======================================

This library provides utility functions for PinSound[^PinSound] sound packs and configuration
files.

This library provides the following functionality:
- [x] Scrambling and unscrambling of sound command IDs.
- [x] Reading encrypted files
- [x] Writing encrypted files

[^PinSound]: *This library is not affiliated, associated, authorized, endorsed by, or in any way
connected with PinSound, SARL, or any of its subsidiaries or its affiliates.
PinSound is a registered trademark of PinSound, SARL.*
*/

mod bytes;
pub mod file;
pub mod scrambler;
pub mod util;
