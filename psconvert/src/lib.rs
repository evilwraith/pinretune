// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Wes Smith

//! PSConvert - PinSound sound packs as AltSound packages
//! =====================================================
//!
//! Reads a PinSound sound pack (group folders, encrypted configuration, scrambled
//! sound folder IDs) and writes it out as a package for
//! [libaltsound](https://github.com/vpinball/libaltsound), the AltSound player used
//! by PinMAME and Visual Pinball standalone:
//!
//! - [`pack`] loads a pack: its configuration, every sound with its command ID,
//!   group, gain and audio files.
//! - [`loudness`] measures how loud the pack's audio is.
//! - [`altsound`] writes an `altsound.csv` or `g-sound.csv` package with its
//!   `altsound.ini`.
//!
//! See the crate README for how PinSound behaviour maps onto each format.
//!
//! *This library is not affiliated, associated, authorized, endorsed by, or in any way
//! connected with PinSound, SARL, or any of its subsidiaries or its affiliates.
//! PinSound is a registered trademark of PinSound, SARL.*

pub mod altsound;
pub mod loudness;
pub mod pack;
