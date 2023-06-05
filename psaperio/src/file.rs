//! Provides a simple interface to work with encrypted PS files.
//!
//! ## Example
//! ``` rust
//! # use psaperio::file;
//! let config = PsConfig::new();
//!_ = config.load("config.pinsound");
//! ```

use std::{
    fs::File,
    io::{Error, ErrorKind as IoErrorKind, Read, Write},
    path::Path,
};

use xsalsa20poly1305::{aead::Aead, AeadInPlace, KeyInit, XSalsa20Poly1305};

// PinSound file encryption key (256bit)
// Note: Key is originally derived from a set of hashing functions and can not be found in binaries
static PS_KEY: &[u8; 32] = include_bytes!("extra/key.bin");

// PinSound file encryption nonce (192bit)
static PS_NONCE: &[u8; 24] = &[
    0x4e, 0x3e, 0x68, 0x72, 0x10, 0xa0, 0xa3, 0x27, 0x5a, 0x00, 0xd2, 0x5b, 0x68, 0xd3, 0xf8, 0xba,
    0x00, 0x43, 0x33, 0x95, 0x90, 0x9e, 0xdb, 0xf7,
];

/// Read and decrypt data from disk
///
/// # Arguments
/// * `filename` - The name of the file to read from
pub fn load<F: AsRef<Path>>(filename: F) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // Decrypt can fail if the key if wrong or data is corrupted
    let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
    match cipher.decrypt_in_place(PS_NONCE.into(), b"", &mut data) {
        Ok(()) => Ok(data),
        Err(_) => Err(Error::new(IoErrorKind::InvalidData, "Decryption failed")),
    }
}

/// Encrypt and write data to disk
///
/// # Arguments
/// * `filename` - The name of the file to write to
/// * `data`- Vec<u8> to encrypt and write
pub fn save<F: AsRef<Path>>(filename: &str, data: &Vec<u8>) -> Result<(), std::io::Error> {
    let mut data = data.clone();
    let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
    cipher
        .encrypt_in_place(PS_NONCE.into(), b"", &mut data)
        .expect("Encryption failure");
    let mut file = File::create(filename)?;
    file.write_all(&data)?;
    Ok(())
}
