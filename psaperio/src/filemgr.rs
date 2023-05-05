use nom::{bytes::complete::take_while, character::complete::char, sequence::terminated};

// A function that will take OsString and split it into two strings on dash delimiter.
// The first string should be all numeric characters, the second string is copied as is.
// The fuction will return an Option<(String, String)>
// If the OsString does not contain a dash delimiter, then the function will return None.
pub fn split_id_str(input: &str) -> Option<(&str, &str)> {
    if let Ok((name, id)) = terminated(take_while(char::is_numeric), char::<_, ()>('-'))(input) {
        Some((id, name))
    } else {
        None
    }
}
