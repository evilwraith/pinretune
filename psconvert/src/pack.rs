// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Wes Smith

//! Loading a PinSound sound pack.
//!
//! A pack is a folder with up to five group folders (`music`, `jingle`, `single`,
//! `sfx`, `voice`; the names are case-sensitive and any other folder is ignored),
//! an encrypted `config.pinsound` and optionally a plaintext `rules.txt` with the
//! same statements one per line. Each sound is a folder `<id>-<label>` inside a
//! group folder, holding one or more audio files (one is picked at random per play).
//!
//! The `<id>` is the sound command sent by the pinball's CPU board. In a pack whose
//! largest ID is above 65535 every ID above that is scrambled with the pack's
//! `checksum` (see [`psaperio::scrambler`]); smaller ones are plain.
//!
//! Gains and ducking can also come from files: `gain.txt` / `ducking.txt` in a group
//! folder set that group's value, `gain.txt` / `delay.txt` in a sound folder that
//! sound's. A `config.pinsound` inside a sound folder holds that sound's `gain` and
//! `delay`. Values in the pack's configuration take precedence over group files.

use psaperio::{file as psfile, scrambler::Scrambler, util};
use psconfig::config::{Kind, PsConfig, SoundAction, Statement};
use psconfig::parse;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration file name, in the pack folder and in sound folders.
pub const PS_CONFIG: &str = "config.pinsound";

/// Group folders in the order sounds are listed.
pub const GROUPS: [Kind; 5] = [
    Kind::Music,
    Kind::Jingle,
    Kind::Single,
    Kind::Sfx,
    Kind::Voice,
];

/// Default ducking (percentage of music/single volume kept) of the groups that duck,
/// as PinSound Studio ships it.
pub fn default_ducking(kind: &Kind) -> Option<u32> {
    match kind {
        Kind::Jingle => Some(5),
        Kind::Sfx => Some(90),
        Kind::Voice => Some(85),
        Kind::Music | Kind::Single => None,
    }
}

/// One sound of a pack.
#[derive(Debug, Clone)]
pub struct Sound {
    /// Sound command ID (unscrambled).
    pub command: u32,
    /// Group the sound belongs to.
    pub kind: Kind,
    /// Sound folder name.
    pub folder: String,
    /// Label part of the folder name.
    pub label: String,
    /// Audio files, sorted by name.
    pub files: Vec<PathBuf>,
    /// The sound's own gain, linear, 10 = unity.
    pub gain: u32,
    /// The sound's start delay in milliseconds, if any.
    pub delay: Option<u32>,
}

/// A loaded sound pack.
#[derive(Debug, Default)]
pub struct Pack {
    /// The pack's configuration (`config.pinsound` followed by `rules.txt`).
    pub config: PsConfig,
    /// Sounds in group order, then folder name order.
    pub sounds: Vec<Sound>,
    /// Statements and files the pack uses that are not part of the sounds or groups
    /// above (rules, polyphony limits, shaker presets, ...), as text.
    pub extras: Vec<String>,
    group_gain_files: Vec<(Kind, u32)>,
    group_ducking_files: Vec<(Kind, u32)>,
}

impl Pack {
    /// Load the pack in `dir`.
    pub fn load(dir: &Path) -> Result<Pack, Box<dyn Error>> {
        let mut pack = Pack::default();

        let config_path = dir.join(PS_CONFIG);
        if config_path.is_file() {
            pack.config = load_config(&config_path)?;
        }
        if let Ok(rules) = fs::read_to_string(dir.join("rules.txt")) {
            for line in rules
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.contains('#'))
            {
                for stmt in parse::parse(line)?.into_statements() {
                    pack.config.push(stmt);
                }
            }
        }
        pack.extras = pack
            .config
            .statements()
            .iter()
            .filter(|s| is_extra(s))
            .map(|s| s.to_string())
            .collect();

        // Folder IDs above 16 bits are scrambled with the checksum
        let mut folders = Vec::new();
        for kind in GROUPS {
            let group_dir = dir.join(kind.to_string());
            if !group_dir.is_dir() {
                continue;
            }
            if let Some(v) = read_number(&group_dir.join("gain.txt")) {
                pack.group_gain_files.push((kind.clone(), v));
            }
            if let Some(v) = read_number(&group_dir.join("ducking.txt")) {
                pack.group_ducking_files.push((kind.clone(), v));
            }
            let mut names: Vec<String> = fs::read_dir(&group_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            names.sort();
            for name in names {
                let Some((id_str, label)) = util::split_id_str(&name) else {
                    continue;
                };
                let Ok(id) = id_str.parse::<u64>() else {
                    continue;
                };
                folders.push((kind.clone(), name.clone(), label.to_string(), id));
            }
        }

        let scrambler = match pack.config.checksum() {
            Some(checksum) => Some(Scrambler::new(checksum)),
            None if folders.iter().any(|f| f.3 > 0xffff) => {
                return Err(
                    "sound folder IDs are scrambled but the configuration has no checksum".into(),
                );
            }
            None => None,
        };

        for (kind, folder, label, id) in folders {
            let command = match (&scrambler, id) {
                (_, id) if id <= 0xffff => id as u32,
                (Some(s), id) if id <= u32::MAX as u64 => s.unscramble(id as u32),
                _ => return Err(format!("sound folder '{folder}' has an invalid ID").into()),
            };
            let sound_dir = dir.join(kind.to_string()).join(&folder);

            let mut gain = read_number(&sound_dir.join("gain.txt"));
            let mut delay = read_number(&sound_dir.join("delay.txt")).filter(|&d| d > 0);
            let sound_config = sound_dir.join(PS_CONFIG);
            if sound_config.is_file() {
                let config = load_config(&sound_config)?;
                gain = config.sound_gain().or(gain);
                delay = config.sound_delay().or(delay);
                for s in config.statements().iter().filter(|s| is_extra(s)) {
                    pack.extras.push(format!("{kind}/{folder}: {s}"));
                }
            }

            let mut files: Vec<PathBuf> = fs::read_dir(&sound_dir)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| is_audio(p))
                .collect();
            files.sort();

            pack.sounds.push(Sound {
                command,
                kind,
                folder,
                label,
                files,
                gain: gain.unwrap_or(10),
                delay,
            });
        }

        Ok(pack)
    }

    /// Gain of a group, linear, 10 = unity.
    pub fn group_gain(&self, kind: &Kind) -> u32 {
        self.config
            .gain(kind)
            .or_else(|| find(&self.group_gain_files, kind))
            .unwrap_or(10)
    }

    /// Ducking of a group that ducks music and single (jingle, sfx, voice): the
    /// percentage of their volume kept while a sound of this group plays.
    pub fn ducking(&self, kind: &Kind) -> Option<u32> {
        let default = default_ducking(kind)?;
        Some(
            self.config
                .ducking(kind)
                .or_else(|| find(&self.group_ducking_files, kind))
                .unwrap_or(default),
        )
    }

    /// Volume of a sound in percent of full scale, as PinSound plays it: the sound's
    /// gain times its group's gain, capped at 100.
    pub fn volume(&self, sound: &Sound) -> u32 {
        let v = 100.0 * sound.gain as f64 / 10.0 * self.group_gain(&sound.kind) as f64 / 10.0;
        (v as i64).clamp(0, 100) as u32
    }
}

fn load_config(path: &Path) -> Result<PsConfig, Box<dyn Error>> {
    let data = psfile::load(path)?;
    Ok(parse::parse(&String::from_utf8_lossy(&data))?)
}

fn read_number(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn find(list: &[(Kind, u32)], kind: &Kind) -> Option<u32> {
    list.iter().find(|(k, _)| k == kind).map(|(_, v)| *v)
}

/// Audio files PinSound plays: WAV, and OGG (converted to WAV on install). Files of
/// 44 bytes or less are empty WAV headers.
fn is_audio(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("wav" | "ogg"))
        && fs::metadata(path).map(|m| m.len() > 44).unwrap_or(false)
}

/// Statements a conversion has to report because it does not carry them over.
fn is_extra(stmt: &Statement) -> bool {
    match stmt {
        Statement::SoundCommand { action, .. } => !matches!(action, SoundAction::Preload),
        Statement::MaxConcurrent { .. } | Statement::Rtttl(_) | Statement::Unknown(_) => true,
        _ => false,
    }
}
