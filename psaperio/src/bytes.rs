//! Byte manipulation
//!

pub trait Halves {
    type Item;

    fn hi(&self) -> Self::Item;
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
