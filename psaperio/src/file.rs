// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Martin Åkesson

//! Provides a simple interface to work with encrypted PS files.
//!
//! ## Example
//! ```rust
//! # use psaperio::file;
//! let _ = file::load("config.pinsound");
//! ```

use std::{
    fs,
    io::{Error, ErrorKind as IoErrorKind, Write},
    path::Path,
};

use crypto_secretbox::{AeadInPlace, KeyInit, XSalsa20Poly1305};

// PinSound file encryption key (256-bit).
// Note: Key is originally derived from a set of hashing functions and cannot be found in binaries.
static PS_KEY: &[u8; 32] = include_bytes!("extra/key.bin");

// PinSound file encryption nonce (192-bit).
static PS_NONCE: &[u8; 24] = &[
    0x4e, 0x3e, 0x68, 0x72, 0x10, 0xa0, 0xa3, 0x27, 0x5a, 0x00, 0xd2, 0x5b, 0x68, 0xd3, 0xf8, 0xba,
    0x00, 0x43, 0x33, 0x95, 0x90, 0x9e, 0xdb, 0xf7,
];

/// Read and decrypt a PinSound file from disk.
///
/// Returns the plaintext bytes on success. Fails with [`std::io::ErrorKind::InvalidData`]
/// if the ciphertext is corrupt or was encrypted with a different key.
pub fn load<F: AsRef<Path>>(filename: F) -> Result<Vec<u8>, std::io::Error> {
    let mut data = fs::read(filename)?;

    let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
    cipher
        .decrypt_in_place(PS_NONCE.into(), b"", &mut data)
        .map_err(|_| Error::new(IoErrorKind::InvalidData, "Decryption failed"))?;

    Ok(data)
}

/// Encrypt and write data to a PinSound file on disk.
///
/// # Arguments
/// * `filename` - Destination path
/// * `data` - Plaintext byte slice to encrypt and write
pub fn save<F: AsRef<Path>>(filename: F, data: &[u8]) -> Result<(), std::io::Error> {
    let mut data = data.to_vec();

    let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
    cipher
        .encrypt_in_place(PS_NONCE.into(), b"", &mut data)
        .map_err(|_| Error::new(IoErrorKind::InvalidData, "Encryption failed"))?;

    let mut file = fs::File::create(filename)?;
    file.write_all(&data)?;
    Ok(())
}
