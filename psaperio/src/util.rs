//! Utility functions for PinSound path and identifier handling.

use nom::{Parser, bytes::complete::take_while, character::complete::char, sequence::terminated};

/// Split a sound folder name of the form `"<digits>-<description>"` into its
/// numeric ID prefix and the description remainder.
///
/// Returns `Some((id_str, description))` where `id_str` is the digit sequence
/// before the first `-` and `description` is everything after it.
/// Returns `None` if the input does not contain a `-` separator or has no
/// leading digits.
///
/// # Examples
/// ```
/// use psaperio::util::split_id_str;
/// assert_eq!(split_id_str("12345-My Theme"), Some(("12345", "My Theme")));
/// assert_eq!(split_id_str("no-dash-prefix"), None);
/// assert_eq!(split_id_str("notanumber"), None);
/// ```
pub fn split_id_str(input: &str) -> Option<(&str, &str)> {
    terminated(take_while(char::is_numeric), char::<_, ()>('-'))
        .parse(input)
        .ok()
        .and_then(|(description, id)| {
            if id.is_empty() {
                None
            } else {
                Some((id, description))
            }
        })
}
