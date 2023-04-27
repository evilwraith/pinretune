
use std::{collections::HashSet,fs::File,io::{Read, Write}};
use xsalsa20poly1305::{AeadInPlace, KeyInit, XSalsa20Poly1305};

// PinSound file encryption key (256bit)
static PS_KEY: &[u8;32] = include_bytes!("extra/key.bin");

// PinSound file encryption nonce (192bit)
static PS_NONCE: &[u8;24] =
    &[0x4e,0x3e,0x68,0x72,0x10,0xa0,0xa3,0x27,0x5a,0x00,0xd2,0x5b,0x68,0xd3,0xf8,0xba,
    0x00,0x43,0x33,0x95,0x90,0x9e,0xdb,0xf7];

pub struct PsConfig {
    elements: HashSet<String>,
    version: u32,
}

impl PsConfig {
    /// Create a new PsConfig struct
    pub fn new() -> Self {
        PsConfig {
            elements: HashSet::new(),
            version: 1,
        }
    }

    pub fn version(self) -> u32 {
        self.version
    }
    /// Read the contents of a PinSound file into a vector of bytes and decrypt
    /// 
    /// # Arguments
    /// * `filename` - The name of the file to read
    pub fn load(mut self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(filename)?;
        let mut data = Vec::new();

        file.read_to_end(&mut data)?;
        let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
        cipher.decrypt_in_place(PS_NONCE.into(), b"", &mut data);

        let cfg_str = String::from_utf8_lossy(&data);
        
        for item in cfg_str.split('|') {
            // String may be null terminated, so trim it
            let item = item.trim_end_matches(char::from(0));
            // Skip empty strings
            if item.is_empty() { continue; };
            println!("Config item: {}, '{}'", item.len(), item.trim());
            self.elements.insert(item.to_string());
        }
        Ok(())
    }

    /// Write the contents of a vector of bytes to a PinSound file and encrypt
    /// 
    /// # Arguments
    /// * `filename` - The name of the file to write
    pub fn save(self, filename: &str, mut data: Vec<u8>) -> Result<(), std::io::Error> {
        let cipher = XSalsa20Poly1305::new_from_slice(PS_KEY).unwrap();
        cipher.encrypt_in_place(PS_NONCE.into(), b"", &mut data);
        let mut file = File::create(filename)?;
        file.write_all(&data)?;
        Ok(())
    }
}