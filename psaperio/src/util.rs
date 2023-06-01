//! Utility functions
//!

use nom::{bytes::complete::take_while, character::complete::char, sequence::terminated};

/// split_id_str convenience function
pub fn split_id_str(input: &str) -> Option<(&str, &str)> {
    if let Ok((name, id)) = terminated(take_while(char::is_numeric), char::<_, ()>('-'))(input) {
        Some((id, name))
    } else {
        None
    }
}
