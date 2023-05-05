//! # Sound ID Scrambler/Unscrambler
//! Module provides a simple interface to scramble/unscramble sound IDs.
//!
//! Sound IDs are encrypted using custom shuffle loops and two 256 byte lookup
//! tables. A static seed and dynamic seed is used to derive the lookup tables.
//! The dynamic seed is generated from the soundpack "checksum" which can be
//! retrieved from the soundpack configuration file.
//!
//! ### Validation
//! There is no way to validate if an id is scrambled or not. It is however possible
//! to make an educated guess. Sound IDs sent by the pinball CPU are 16bit unsigned
//! and as such will at most be 5 digits long (0-65535). Therefore any id having 6 or
//! more digits can be considered scrambled.
//!
//! Furthermore, unscrambling a scrambled id with the correct checksum seed will
//! always yield a id less than 65536.
//! Any id greater than 4294967295 (32bit 0xffffffff) should be considered invalid.
//!
//! ## Example:
//! ```
//! # use psaperio::scrambler::Scrambler;
//! let scrambler = Scrambler::new("checksum");
//! let new_id = scrambler.scramble(12345);
//! ```

use super::bytes::Halves;
use generic_array::GenericArray;
use sha2::{Digest, Sha256};

/// PinSound ID static encryption seed (256bit)
/// Although the seed is not a encryption key by itself, it is used, together with a
/// appended dynamic part, to to derive lookup tables for use in ID scrambling.
static PS_SEED: &[u8; 32] = include_bytes!("extra/seed.bin");

/// initialise_seed - Initialise seed with soundpack "checksum" for ID scrambling  
fn initialise_seed(checksum: &str) -> [u8; 48] {
    let mut seed: [u8; 48] = [0; 48];

    let mut preseed: Vec<u8> = PS_SEED.to_vec();
    preseed.extend_from_slice(checksum.as_bytes());

    let mut hasher = Sha256::new();
    hasher.update(&preseed);
    let ga_seed = GenericArray::from_mut_slice(&mut seed[0..32]);
    hasher.finalize_into_reset(ga_seed);

    hasher.update(&seed[0..32]);
    hasher.update(&preseed);
    let ga_seed = GenericArray::from_mut_slice(&mut seed[16..48]);
    hasher.finalize_into(ga_seed);

    seed
}

/// mutate_seed - Mutate seed with its own hash and return a u32 for use in ID scrambling
fn mutate_seed(seed: &mut [u8; 48]) -> u32 {
    let mut hash: [u8; 32] = [0; 32];
    let mut ga_hash = GenericArray::from_mut_slice(&mut hash[..]);
    let mut hasher = Sha256::new();

    hasher.update(&seed);
    hasher.finalize_into(&mut ga_hash);

    let offset = seed[0x2f] as usize & 0xf;
    seed[0x2f] = seed[0x2f].wrapping_add(1);

    for i in 0..32 as usize {
        seed[offset + i] ^= hash[i];
    }

    hash[0..4].iter().fold(0, |acc, &x| acc << 8 | x as u32)
}

/// generate_data - Return a `size` long lookup table used for ID scrambling
fn generate_data(seed: &mut [u8; 48], size: usize) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(size);
    let mut hbuf: [u8; 32] = [0; 32];
    let mut ga_hbuf = GenericArray::from_mut_slice(&mut hbuf);

    let mut hasher = Sha256::new();
    for _i in 0..size {
        hasher.update(&seed);
        hasher.finalize_into_reset(&mut ga_hbuf);

        let offset: usize = seed[0x2f] as usize & 0xf;
        seed[0x2f] = seed[0x2f].wrapping_add(1);

        for i in 0..32 {
            seed[offset + i] ^= ga_hbuf[i];
        }

        // In the original code this byte is taken from the first byte of the private sha256 state
        // field and will have endianess issues. This is slightly puzzling as the finalize call will
        // output the state in big endian so it would have made sense to use that for portability.
        // In any case, this is why we fetch the fourth byte to replicate the original code.
        result.push(ga_hbuf[3]);
    }
    result
}

struct IdLookupTable {
    offset1: u32,
    offset2: u32,
    lookup1: Vec<u8>,
    lookup2: Vec<u8>,
}

impl IdLookupTable {
    fn new(checksum: &str) -> Self {
        let mut seed = initialise_seed(checksum);
        IdLookupTable {
            offset1: mutate_seed(&mut seed),
            offset2: mutate_seed(&mut seed),
            lookup1: generate_data(&mut seed, 256),
            lookup2: generate_data(&mut seed, 256),
        }
    }
}

/// Sound ID scrambler
pub struct Scrambler {
    tbl: IdLookupTable,
}

impl Scrambler {
    /// Initialise and return a new ID scrambler with the given dynamic "checksum" seed.
    pub fn new(checksum: &str) -> Self {
        Scrambler {
            tbl: IdLookupTable::new(checksum),
        }
    }

    /// Scramble the given ID and return the result. *This function does no
    /// validation of the ID before or after scrambling.*
    pub fn scramble(&self, id: u32) -> u32 {
        let mut upper = id.hi().wrapping_add(self.tbl.offset1.lo());
        let mut lower = id.lo().wrapping_add(self.tbl.offset2.lo());
        let tbl1 = &self.tbl.lookup1;
        let tbl2 = &self.tbl.lookup2;

        for round in 0..12 as u8 {
            let ih: u8 = upper.hi();
            let il: u8 = upper.lo() ^ round;
            let bvar1 = tbl2[(tbl1[ih as usize] ^ il) as usize];
            let bvar2 = tbl2[(tbl1[il as usize] ^ ih ^ round) as usize].wrapping_add(bvar1);
            let ih = tbl2[(tbl1[bvar2 as usize] ^ bvar1 ^ round) as usize].wrapping_sub(ih);
            let bvar3 = tbl1[ih as usize] ^ bvar2 ^ round;
            let uvar4 = (ih as u16) << 8 | tbl1[bvar2 as usize] as u16;
            let uvar5: u16 = (((tbl2[(tbl1[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl2[((tbl2[bvar3 as usize].wrapping_add(ih)) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            lower = lower.wrapping_add(uvar5);

            let ih = lower.hi();
            let il = lower.lo() ^ round;
            let bvar1 = tbl1[(tbl2[ih as usize] ^ il) as usize];
            let bvar2 = tbl1[(tbl2[il as usize] ^ ih ^ round) as usize].wrapping_add(bvar1);
            let ih = tbl1[(tbl2[bvar2 as usize] ^ bvar1 ^ round) as usize].wrapping_sub(ih);
            let bvar3 = tbl2[ih as usize] ^ bvar2 ^ round;
            let uvar4 = (ih as u16) << 8 | tbl2[bvar2 as usize] as u16;
            let uvar5 = (((tbl1[(tbl2[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl1[(tbl1[bvar3 as usize].wrapping_add(ih) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            upper = upper.wrapping_sub(uvar5);
        }

        (upper as u32) << 16 | lower as u32
    }

    /// Unscramble the given ID and return the result. *This function does no
    /// validation of the ID before or after unscrambling.*
    pub fn unscramble(&self, id: u32) -> u32 {
        let mut upper = id.hi();
        let mut lower = id.lo();
        let tbl1 = &self.tbl.lookup1;
        let tbl2 = &self.tbl.lookup2;

        for round in (0..12 as u8).rev() {
            let ih = lower.hi();
            let il = lower.lo() ^ round;
            let bvar1 = tbl1[(tbl2[ih as usize] ^ il) as usize];
            let bvar2 = tbl1[(tbl2[il as usize] ^ ih ^ round) as usize].wrapping_add(bvar1);
            let ih = tbl1[(tbl2[bvar2 as usize] ^ bvar1 ^ round) as usize].wrapping_sub(ih);
            let bvar3 = tbl2[ih as usize] ^ bvar2 ^ round;
            let uvar4 = (ih as u16) << 8 | tbl2[bvar2 as usize] as u16;
            let uvar5 = (((tbl1[(tbl2[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl1[(tbl1[bvar3 as usize].wrapping_add(ih) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            upper = upper.wrapping_add(uvar5);

            let ih: u8 = upper.hi();
            let il: u8 = upper.lo() ^ round;
            let bvar1 = tbl2[(tbl1[ih as usize] ^ il) as usize];
            let bvar2 = tbl2[(tbl1[il as usize] ^ ih ^ round) as usize].wrapping_add(bvar1);
            let ih = tbl2[(tbl1[bvar2 as usize] ^ bvar1 ^ round) as usize].wrapping_sub(ih);
            let bvar3 = tbl1[ih as usize] ^ bvar2 ^ round;
            let uvar4 = (ih as u16) << 8 | tbl1[bvar2 as usize] as u16;
            let uvar5: u16 = (((tbl2[(tbl1[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl2[((tbl2[bvar3 as usize].wrapping_add(ih)) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            lower = lower.wrapping_sub(uvar5);
        }

        upper = upper.wrapping_sub(self.tbl.offset1.lo());
        lower = lower.wrapping_sub(self.tbl.offset2.lo());
        (upper as u32) << 16 | lower as u32
    }
}

#[cfg(test)]
mod tests {
    use super::Scrambler;

    #[test]
    fn test_revscramble() {
        let s = Scrambler::new("Pinball");
        assert_eq!(s.unscramble(s.scramble(777)), 777);
    }

    #[test]
    fn test_sttng() {
        let s = Scrambler::new("JeanLucPicard");
        assert_eq!(s.scramble(10), 2824050793); // Multiball theme
    }

    #[test]
    fn test_tzremix() {
        let s = Scrambler::new("TZPSRemix");
        assert_eq!(s.scramble(10), 1098459480); // Multiball theme
    }
}
