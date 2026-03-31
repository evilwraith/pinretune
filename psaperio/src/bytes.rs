//! Byte-level manipulation utilities.
//!
//! Provides the [`Halves`] trait for splitting integer types into their
//! high and low halves — for example, splitting a `u32` into two `u16`s
//! or a `u16` into two `u8`s.

/// Split an integer into its high and low halves.
///
/// The high half contains the most-significant bits and the low half
/// the least-significant bits, both of half the original width.
pub trait Halves {
    /// The type of each half (half the bit-width of `Self`).
    type Item;

    /// Returns the most-significant (upper) half.
    fn hi(&self) -> Self::Item;

    /// Returns the least-significant (lower) half.
    fn lo(&self) -> Self::Item;
}

impl Halves for u16 {
    type Item = u8;

    fn hi(&self) -> Self::Item {
        (*self >> 8) as u8
    }

    fn lo(&self) -> Self::Item {
        *self as u8
    }
}

impl Halves for u32 {
    type Item = u16;

    fn hi(&self) -> Self::Item {
        (*self >> 16) as u16
    }

    fn lo(&self) -> Self::Item {
        *self as u16
    }
}

#[cfg(test)]
mod tests {
    use super::Halves;

    #[test]
    fn test_u32() {
        let val: u32 = 0x1234abcd;
        assert_eq!(val.hi(), 0x1234);
        assert_eq!(val.lo(), 0xabcd);
    }

    #[test]
    fn test_u16() {
        let val: u16 = 0x12ab;
        assert_eq!(val.hi(), 0x12);
        assert_eq!(val.lo(), 0xab);
    }
}
