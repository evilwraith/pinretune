//! Typed configuration model for PinSound configuration files.
//!
//! A configuration file consists of pipe-delimited [`Statement`]s. Each statement
//! is either a known configuration primitive, a sound command, or an unknown
//! command preserved verbatim for round-trip fidelity.
//!
//! # Adding new commands
//!
//! To add support for a new command:
//! 1. Add a variant to [`Statement`] (or [`SoundAction`] for sound commands).
//! 2. Implement its [`fmt::Display`] arm to serialize back to config format.
//! 3. Add a match arm in [`crate::parse::parse_statement`] (or
//!    [`crate::parse::parse_sound_command`]) to parse it.

use std::fmt;
use std::str::FromStr;

/// Sound category kind used by `max` and `ducking` statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Jingle,
    Music,
    Sfx,
    Single,
    Voice,
}

/// The error type returned when a string cannot be parsed into a [`Kind`].
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownKind;

impl fmt::Display for UnknownKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown sound category kind")
    }
}

impl std::error::Error for UnknownKind {}

impl FromStr for Kind {
    type Err = UnknownKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jingle" => Ok(Kind::Jingle),
            "music" => Ok(Kind::Music),
            "sfx" => Ok(Kind::Sfx),
            "single" => Ok(Kind::Single),
            "voice" => Ok(Kind::Voice),
            _ => Err(UnknownKind),
        }
    }
}

impl Kind {
    /// All known kind values, useful for iteration.
    pub const ALL: [Kind; 5] = [
        Kind::Jingle,
        Kind::Music,
        Kind::Sfx,
        Kind::Single,
        Kind::Voice,
    ];
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Jingle => write!(f, "jingle"),
            Kind::Music => write!(f, "music"),
            Kind::Sfx => write!(f, "sfx"),
            Kind::Single => write!(f, "single"),
            Kind::Voice => write!(f, "voice"),
        }
    }
}

/// An action associated with a sound command (`<id> <action> [args]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundAction {
    /// Cancel playback of another sound: `<id> stop <target_id>`
    Stop(u32),
    /// Delay before action in milliseconds: `<id> delay <ms>`
    Delay(u32),
    /// Trigger shaker preset: `<id> shakerPreset <preset>`
    ShakerPreset(u32),
    /// Preload a sound: `<id> preload`
    Preload,
}

impl fmt::Display for SoundAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoundAction::Stop(id) => write!(f, "stop {id}"),
            SoundAction::Delay(ms) => write!(f, "delay {ms}"),
            SoundAction::ShakerPreset(p) => write!(f, "shakerPreset {p}"),
            SoundAction::Preload => write!(f, "preload"),
        }
    }
}

/// A single statement in a PinSound configuration file.
///
/// Each variant maps to a known command keyword. Unrecognised commands are
/// captured in [`Statement::Unknown`] so the file can be round-tripped without
/// data loss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// Application version: `versionDetail <version>`
    AppVersion(String),
    /// Checksum seed for ID scrambling: `checksum <seed>`
    Checksum(String),
    /// List of valid checksums: `checksums <name> [<name> ...]`
    Checksums(Vec<String>),
    /// Checksum format version: `checksum_version <n>`
    ChecksumVersion(u32),
    /// Config file format version: `versionConfigFile <n>`
    ConfigVersion(u32),
    /// Maximum concurrent sounds per category: `max <kind> <n>`
    MaxConcurrent { kind: Kind, count: u32 },
    /// Creation timestamp (Unix epoch): `timestamp <epoch>`
    Timestamp(u32),
    /// Audio ducking level per category: `ducking <kind> <level>`
    Ducking { kind: Kind, level: u32 },
    /// Sound pack identifier (hex string): `pspack_id <hex>`
    PackId(String),
    /// Recording enabled flag: `record_enabled <0|1>`
    RecordEnabled(u32),
    /// RTTTL ringtone data: `rtttl <data>`
    Rtttl(String),
    /// Sound command: `<id> <action> [args]`
    SoundCommand { id: u32, action: SoundAction },
    /// Unrecognised statement, preserved verbatim for round-trip fidelity.
    Unknown(String),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::AppVersion(v) => write!(f, "versionDetail {v}"),
            Statement::Checksum(v) => write!(f, "checksum {v}"),
            Statement::Checksums(v) => write!(f, "checksums {}", v.join(" ")),
            Statement::ChecksumVersion(v) => write!(f, "checksum_version {v}"),
            Statement::ConfigVersion(v) => write!(f, "versionConfigFile {v}"),
            Statement::MaxConcurrent { kind, count } => write!(f, "max {kind} {count}"),
            Statement::Timestamp(v) => write!(f, "timestamp {v}"),
            Statement::Ducking { kind, level } => write!(f, "ducking {kind} {level}"),
            Statement::PackId(v) => write!(f, "pspack_id {v}"),
            Statement::RecordEnabled(v) => write!(f, "record_enabled {v}"),
            Statement::Rtttl(v) => write!(f, "rtttl {v}"),
            Statement::SoundCommand { id, action } => write!(f, "{id} {action}"),
            Statement::Unknown(v) => write!(f, "{v}"),
        }
    }
}

/// Parsed PinSound configuration file.
///
/// Maintains statement order for round-trip serialization fidelity.
/// Provides typed accessors for commonly needed values.
///
/// ## Example
/// ```
/// use psconfig::config::{PsConfig, Statement};
///
/// let mut config = PsConfig::new();
/// config.push(Statement::Checksum("JeanLucPicard".to_string()));
/// assert_eq!(config.checksum(), Some("JeanLucPicard"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct PsConfig {
    statements: Vec<Statement>,
}

impl PsConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the checksum seed value used for ID scrambling.
    pub fn checksum(&self) -> Option<&str> {
        self.statements.iter().find_map(|s| match s {
            Statement::Checksum(v) => Some(v.as_str()),
            _ => None,
        })
    }

    /// Get the list of valid checksums.
    pub fn checksums(&self) -> Option<&[String]> {
        self.statements.iter().find_map(|s| match s {
            Statement::Checksums(v) => Some(v.as_slice()),
            _ => None,
        })
    }

    /// Get the sound pack identifier.
    pub fn pack_id(&self) -> Option<&str> {
        self.statements.iter().find_map(|s| match s {
            Statement::PackId(v) => Some(v.as_str()),
            _ => None,
        })
    }

    /// Get the application version string.
    pub fn app_version(&self) -> Option<&str> {
        self.statements.iter().find_map(|s| match s {
            Statement::AppVersion(v) => Some(v.as_str()),
            _ => None,
        })
    }

    /// Iterate over all sound commands as `(id, action)` pairs.
    pub fn sound_commands(&self) -> impl Iterator<Item = (u32, &SoundAction)> {
        self.statements.iter().filter_map(|s| match s {
            Statement::SoundCommand { id, action } => Some((*id, action)),
            _ => None,
        })
    }

    /// Borrow the ordered list of statements.
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    /// Take ownership of the statement list.
    pub fn into_statements(self) -> Vec<Statement> {
        self.statements
    }

    /// Append a statement to the configuration.
    pub fn push(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    /// Serialize back to pipe-delimited configuration format.
    ///
    /// Each statement is followed by a `|` delimiter (including the last one),
    /// matching the format used by PinSound firmware.
    pub fn to_config_string(&self) -> String {
        let mut out = String::new();
        for stmt in &self.statements {
            out.push_str(&stmt.to_string());
            out.push('|');
        }
        out
    }
}
