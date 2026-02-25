//! Parser for PinSound configuration files.
//!
//! Configuration files consist of pipe-delimited statements. Each statement
//! begins with a keyword (or numeric sound ID) followed by optional arguments
//! separated by whitespace.
//!
//! ## Example
//! ```rust
//! use psconfig::parse;
//!
//! let input = "checksum_version 1|checksum JeanLucPicard|355 stop 105|";
//! let config = parse::parse(input).unwrap();
//! assert_eq!(config.checksum(), Some("JeanLucPicard"));
//! ```

use crate::config::{Kind, PsConfig, SoundAction, Statement};
use crate::error::ParseError;

/// Parse a PinSound configuration string into a [`PsConfig`].
///
/// Splits the input on `|` delimiters and parses each segment as a
/// [`Statement`]. Unknown commands are preserved verbatim.
pub fn parse(input: &str) -> Result<PsConfig, ParseError> {
    let mut config = PsConfig::new();

    for segment in input.split('|') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        // Fall back to Unknown for malformed statements rather than
        // rejecting the entire configuration file.
        let stmt = match parse_statement(segment) {
            Ok(s) => s,
            Err(_) => Statement::Unknown(segment.to_string()),
        };
        config.push(stmt);
    }

    Ok(config)
}

/// Parse a single statement (text between pipe delimiters).
fn parse_statement(input: &str) -> Result<Statement, ParseError> {
    let (keyword, rest) = split_first_token(input);

    // Leading digits → sound command
    if !keyword.is_empty() && keyword.chars().all(|c| c.is_ascii_digit()) {
        return parse_sound_command(keyword, rest);
    }

    match keyword {
        "versionDetail" => {
            require_value("versionDetail", rest).map(|v| Statement::AppVersion(v.to_string()))
        }
        "checksum" => require_value("checksum", rest).map(|v| Statement::Checksum(v.to_string())),
        "checksums" => parse_checksums(rest),
        "checksum_version" => parse_u32("checksum_version", rest).map(Statement::ChecksumVersion),
        "versionConfigFile" => parse_u32("versionConfigFile", rest).map(Statement::ConfigVersion),
        "timestamp" => parse_u32("timestamp", rest).map(Statement::Timestamp),
        "pspack_id" => require_value("pspack_id", rest).map(|v| Statement::PackId(v.to_string())),
        "record_enabled" => parse_u32("record_enabled", rest).map(Statement::RecordEnabled),
        "max" => parse_kind_u32("max", rest)
            .map(|(kind, count)| Statement::MaxConcurrent { kind, count }),
        "ducking" => {
            parse_kind_u32("ducking", rest).map(|(kind, level)| Statement::Ducking { kind, level })
        }
        "rtttl" => require_value("rtttl", rest).map(|v| Statement::Rtttl(v.to_string())),
        _ => Ok(Statement::Unknown(input.to_string())),
    }
}

/// Parse a sound command: `<id> <action> [args]`
fn parse_sound_command(id_str: &str, rest: &str) -> Result<Statement, ParseError> {
    let id: u32 = id_str.parse().map_err(|_| ParseError::InvalidValue {
        field: "sound command id",
        value: id_str.to_string(),
    })?;

    let (action_kw, action_rest) = split_first_token(rest);

    let action = match action_kw {
        "stop" => SoundAction::Stop(parse_u32("stop target", action_rest)?),
        "delay" => SoundAction::Delay(parse_u32("delay ms", action_rest)?),
        "shakerPreset" => SoundAction::ShakerPreset(parse_u32("shakerPreset", action_rest)?),
        "preload" => SoundAction::Preload,
        _ => {
            // Unknown sound action — preserve the entire segment verbatim
            let full = if rest.is_empty() {
                id_str.to_string()
            } else {
                format!("{} {}", id_str, rest)
            };
            return Ok(Statement::Unknown(full));
        }
    };

    Ok(Statement::SoundCommand { id, action })
}

// ── helper parsers ──────────────────────────────────────────────────────────

/// Split input into the first whitespace-delimited token and the remainder.
fn split_first_token(input: &str) -> (&str, &str) {
    match input.split_once(char::is_whitespace) {
        Some((first, rest)) => (first, rest.trim_start()),
        None => (input, ""),
    }
}

/// Parse a `<kind> <u32>` pair (used by `max` and `ducking`).
fn parse_kind_u32(field: &'static str, input: &str) -> Result<(Kind, u32), ParseError> {
    let (kind_str, value_str) = split_first_token(input);

    let kind = Kind::parse(kind_str).ok_or_else(|| ParseError::InvalidValue {
        field,
        value: kind_str.to_string(),
    })?;

    let value = parse_u32(field, value_str)?;
    Ok((kind, value))
}

/// Parse whitespace-separated checksum names.
fn parse_checksums(input: &str) -> Result<Statement, ParseError> {
    if input.is_empty() {
        return Err(ParseError::MissingValue { field: "checksums" });
    }
    let names: Vec<String> = input.split_whitespace().map(String::from).collect();
    Ok(Statement::Checksums(names))
}

/// Parse the first token as a `u32`.
fn parse_u32(field: &'static str, input: &str) -> Result<u32, ParseError> {
    if input.is_empty() {
        return Err(ParseError::MissingValue { field });
    }
    let token = input.split_whitespace().next().unwrap_or(input);
    token.parse::<u32>().map_err(|_| ParseError::InvalidValue {
        field,
        value: token.to_string(),
    })
}

/// Require a non-empty value string.
fn require_value<'a>(field: &'static str, input: &'a str) -> Result<&'a str, ParseError> {
    if input.is_empty() {
        Err(ParseError::MissingValue { field })
    } else {
        Ok(input)
    }
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Example config from PSConfig README.
    const EXAMPLE: &str = "\
        checksum_version 1|pspack_id b00bfaceb00bface|checksum JeanLucPicard|\
        355 stop 105 |500 stop 374 |700 stop 374 |703 stop 374 |\
        max sfx 3|max voice 2|\
        ducking sfx 90|ducking voice 85|ducking jingle 5|\
        record_enabled 1|0 preload |\
        timestamp 1681718217|versionDetail 22.05.1|versionConfigFile 1|";

    #[test]
    fn parse_full_example() {
        let config = parse(EXAMPLE).unwrap();
        assert_eq!(config.checksum(), Some("JeanLucPicard"));
        assert_eq!(config.pack_id(), Some("b00bfaceb00bface"));
        assert_eq!(config.app_version(), Some("22.05.1"));
        assert_eq!(config.statements().len(), 17);
    }

    #[test]
    fn parse_checksum_version() {
        let config = parse("checksum_version 1|").unwrap();
        assert_eq!(config.statements()[0], Statement::ChecksumVersion(1));
    }

    #[test]
    fn parse_sound_stop() {
        let config = parse("355 stop 105|").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::SoundCommand {
                id: 355,
                action: SoundAction::Stop(105)
            }
        );
    }

    #[test]
    fn parse_max_concurrent() {
        let config = parse("max sfx 3|").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::MaxConcurrent {
                kind: Kind::Sfx,
                count: 3
            }
        );
    }

    #[test]
    fn parse_ducking() {
        let config = parse("ducking voice 85|").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::Ducking {
                kind: Kind::Voice,
                level: 85
            }
        );
    }

    #[test]
    fn parse_preload() {
        let config = parse("0 preload|").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::SoundCommand {
                id: 0,
                action: SoundAction::Preload
            }
        );
    }

    #[test]
    fn unknown_statements_preserved() {
        let config = parse("auto_mute 600|donotstop|").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::Unknown("auto_mute 600".to_string())
        );
        assert_eq!(
            config.statements()[1],
            Statement::Unknown("donotstop".to_string())
        );
    }

    #[test]
    fn roundtrip() {
        let config = parse(EXAMPLE).unwrap();
        let serialized = config.to_config_string();
        let reparsed = parse(&serialized).unwrap();
        assert_eq!(config.statements().len(), reparsed.statements().len());
        for (a, b) in config.statements().iter().zip(reparsed.statements().iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn missing_value_falls_back_to_unknown() {
        // A malformed statement (missing value) is preserved as Unknown
        // rather than failing the entire parse.
        let config = parse("checksum |").unwrap();
        assert_eq!(
            config.statements()[0],
            Statement::Unknown("checksum".to_string())
        );
        assert!(config.checksum().is_none());
    }
}
