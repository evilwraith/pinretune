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
use sha2::digest::{FixedOutput, FixedOutputReset};
use sha2::{Digest, Sha256};

/// PinSound ID static scrambler seed (256‑bit).
/// This is compiled in once and reused when a dynamic checksum is supplied.
const PS_SEED: [u8; 32] = *include_bytes!("extra/seed.bin");

/// Number of rounds in the Feistel-like permutation network.
/// Matches the value used by the original PinSound firmware.
const ROUNDS: usize = 12;

/// Internal generator state used during the "key schedule" phase.
///
/// The original C implementation manipulated a 48‑byte buffer directly; we
/// wrap that buffer here and provide a couple of small helpers so that the
/// rest of the module can be expressed in more idiomatic Rust.  The
/// behaviour is identical to the reverse‑engineered code.
struct Seed([u8; 48]);

impl Seed {
    /// Create a new state from the static seed plus the user‑supplied
    /// checksum string.
    fn new(checksum: &str) -> Self {
        let mut result = [0u8; 48];

        // preseed = PS_SEED || checksum
        let mut preseed = Vec::with_capacity(PS_SEED.len() + checksum.len());
        preseed.extend_from_slice(&PS_SEED);
        preseed.extend_from_slice(checksum.as_bytes());

        let mut hasher = Sha256::new();

        hasher.update(&preseed);
        let out = hasher.finalize_fixed_reset();
        result[..32].copy_from_slice(&out);

        hasher.update(out);
        hasher.update(&preseed);
        let out = hasher.finalize_fixed();
        result[16..].copy_from_slice(&out);

        Seed(result)
    }

    /// Advance the state by one step and return the resulting hash bytes.
    ///
    /// This is the core "clock" of the key schedule: hash the current buffer,
    /// XOR the output back in at a rolling offset, then return the raw hash so
    /// callers can extract whatever bytes they need from it.
    fn step(&mut self) -> [u8; 32] {
        let hash = Sha256::digest(self.0);

        let offset = (self.0[0x2f] as usize) & 0xf;
        self.0[0x2f] = self.0[0x2f].wrapping_add(1);

        for (i, &b) in hash.iter().enumerate() {
            self.0[offset + i] ^= b;
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&hash);
        out
    }

    /// Advance the state and return a 32‑bit word from the first four hash bytes.
    fn next_offset(&mut self) -> u32 {
        let hash = self.step();
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Produce a 256‑element lookup table, advancing the state once per entry.
    fn generate_table(&mut self) -> [u8; 256] {
        let mut table = [0u8; 256];
        for entry in table.iter_mut() {
            // The fourth byte of each hash step matches the original implementation.
            *entry = self.step()[3];
        }
        table
    }
}

struct IdLookupTable {
    offset1: u32,
    offset2: u32,
    lookup1: [u8; 256],
    lookup2: [u8; 256],
}

impl IdLookupTable {
    fn new(checksum: &str) -> Self {
        let mut seed = Seed::new(checksum);
        IdLookupTable {
            offset1: seed.next_offset(),
            offset2: seed.next_offset(),
            lookup1: seed.generate_table(),
            lookup2: seed.generate_table(),
        }
    }
}

/// Sound ID scrambler/unscrambler.  The transformation is deterministic and
/// reversible; the `checksum` string is effectively the key material.
pub struct Scrambler {
    tbl: IdLookupTable,
}

impl Scrambler {
    /// Create a new scrambler keyed by the provided checksum.
    pub fn new(checksum: &str) -> Self {
        Scrambler {
            tbl: IdLookupTable::new(checksum),
        }
    }

    /// Apply the forward permutation.  No range checks are performed.
    pub fn scramble(&self, id: u32) -> u32 {
        let mut upper = id.hi().wrapping_add(self.tbl.offset1.lo());
        let mut lower = id.lo().wrapping_add(self.tbl.offset2.lo());
        let tbl1 = &self.tbl.lookup1;
        let tbl2 = &self.tbl.lookup2;

        for round in 0..ROUNDS {
            // mix upper into lower
            let ih = upper.hi();
            let il = upper.lo() ^ round as u8;
            let bvar1 = tbl2[(tbl1[ih as usize] ^ il) as usize];
            let bvar2 = tbl2[(tbl1[il as usize] ^ ih ^ round as u8) as usize].wrapping_add(bvar1);
            let ih = tbl2[(tbl1[bvar2 as usize] ^ bvar1 ^ round as u8) as usize].wrapping_sub(ih);
            let bvar3 = tbl1[ih as usize] ^ bvar2 ^ round as u8;
            let uvar4 = (ih as u16) << 8 | tbl1[bvar2 as usize] as u16;
            let uvar5: u16 = (((tbl2[(tbl1[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl2[((tbl2[bvar3 as usize].wrapping_add(ih)) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            lower = lower.wrapping_add(uvar5);

            // mix lower back into upper
            let ih = lower.hi();
            let il = lower.lo() ^ round as u8;
            let bvar1 = tbl1[(tbl2[ih as usize] ^ il) as usize];
            let bvar2 = tbl1[(tbl2[il as usize] ^ ih ^ round as u8) as usize].wrapping_add(bvar1);
            let ih = tbl1[(tbl2[bvar2 as usize] ^ bvar1 ^ round as u8) as usize].wrapping_sub(ih);
            let bvar3 = tbl2[ih as usize] ^ bvar2 ^ round as u8;
            let uvar4 = (ih as u16) << 8 | tbl2[bvar2 as usize] as u16;
            let uvar5 = (((tbl1[(tbl2[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl1[(tbl1[bvar3 as usize].wrapping_add(ih) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            upper = upper.wrapping_sub(uvar5);
        }

        (upper as u32) << 16 | lower as u32
    }

    /// Inverse of `scramble`.
    pub fn unscramble(&self, id: u32) -> u32 {
        let mut upper = id.hi();
        let mut lower = id.lo();
        let tbl1 = &self.tbl.lookup1;
        let tbl2 = &self.tbl.lookup2;

        for round in (0..ROUNDS).rev() {
            // undo lower→upper then upper→lower sequence
            let ih = lower.hi();
            let il = lower.lo() ^ round as u8;
            let bvar1 = tbl1[(tbl2[ih as usize] ^ il) as usize];
            let bvar2 = tbl1[(tbl2[il as usize] ^ ih ^ round as u8) as usize].wrapping_add(bvar1);
            let ih = tbl1[(tbl2[bvar2 as usize] ^ bvar1 ^ round as u8) as usize].wrapping_sub(ih);
            let bvar3 = tbl2[ih as usize] ^ bvar2 ^ round as u8;
            let uvar4 = (ih as u16) << 8 | tbl2[bvar2 as usize] as u16;
            let uvar5 = (((tbl1[(tbl2[bvar1 as usize] ^ ih) as usize] ^ bvar2) as u16) << 8)
                | (tbl1[(tbl1[bvar3 as usize].wrapping_add(ih) ^ uvar4 as u8) as usize] ^ bvar1)
                    as u16;
            upper = upper.wrapping_add(uvar5);

            let ih = upper.hi();
            let il = upper.lo() ^ round as u8;
            let bvar1 = tbl2[(tbl1[ih as usize] ^ il) as usize];
            let bvar2 = tbl2[(tbl1[il as usize] ^ ih ^ round as u8) as usize].wrapping_add(bvar1);
            let ih = tbl2[(tbl1[bvar2 as usize] ^ bvar1 ^ round as u8) as usize].wrapping_sub(ih);
            let bvar3 = tbl1[ih as usize] ^ bvar2 ^ round as u8;
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
