//! Error types for PinSound configuration parsing.

use std::fmt;

/// Errors that can occur while parsing a PinSound configuration file.
#[derive(Debug)]
pub enum ParseError {
    /// A required value is missing for a known field.
    MissingValue { field: &'static str },
    /// A value could not be parsed or is not valid for the given field.
    InvalidValue { field: &'static str, value: String },
    /// An I/O error occurred while reading or writing a configuration file.
    Io(std::io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingValue { field } => {
                write!(f, "missing value for '{}'", field)
            }
            ParseError::InvalidValue { field, value } => {
                write!(f, "invalid value '{}' for '{}'", value, field)
            }
            ParseError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::Io(e)
    }
}
