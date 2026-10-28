// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Wes Smith

//! Writing a sound pack as an AltSound package.
//!
//! libaltsound reads two CSV formats; both are written with an `altsound.ini`:
//!
//! - **AltSound** (`altsound.csv`): each sample has a channel (0 music, 1 jingle,
//!   -1 everything else), ducking, gain, loop and a stop-music flag.
//! - **G-Sound** (`g-sound.csv`): each sample has a type (MUSIC, CALLOUT, SFX, SOLO,
//!   OVERLAY) whose behaviour, including who ducks or stops whom, is set in the ini.
//!   PinSound's five groups map onto the five types one to one, so this is the
//!   closer match.
//!
//! PinSound behaviour carried over:
//!
//! | PinSound | AltSound | G-Sound |
//! |---|---|---|
//! | music: one at a time, loops, stops a single | channel 0, LOOP | MUSIC, `stops = solo` |
//! | single: one at a time, stops music | channel 1, STOP | SOLO, `stops = music` |
//! | jingle: one at a time, ducks music and single | channel 1, DUCK | OVERLAY, ducking profile |
//! | voice: ducks music and single | channel -1, DUCK | CALLOUT, ducking profile |
//! | sfx: ducks music and single | channel -1, DUCK | SFX, ducking profile |
//!
//! Not carried over by either format: the `max` polyphony limits (2 voices, 3 sfx),
//! the gradual ducking ramp, start delays, `<id> stop/donotstop/playif` rules and the
//! Data East music stack. [`convert`] reports what a pack uses of these.
//!
//! A sample's gain is capped at 100 in both formats, so the pack's level goes into
//! `altsound.ini` as `package_gain_db` (libaltsound with package gain support;
//! older versions ignore the key).

use crate::loudness;
use crate::pack::{Pack, Sound};
use psconfig::config::Kind;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// `altsound.csv`
    AltSound,
    /// `g-sound.csv`
    GSound,
}

/// Conversion options.
#[derive(Debug, Clone)]
pub struct Options {
    pub format: Format,
    /// Average loudness in dBFS to raise the pack to, or `None` to leave its level.
    /// A pack is never lowered.
    pub target_db: Option<f64>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            format: Format::AltSound,
            target_db: Some(-19.0),
        }
    }
}

/// What a conversion did.
#[derive(Debug, Default)]
pub struct Report {
    /// Number of samples (audio files) written.
    pub samples: usize,
    /// Number of distinct sound commands.
    pub commands: usize,
    /// Average loudness of the pack, if measured.
    pub loudness_db: Option<f64>,
    /// Gain written as `package_gain_db`.
    pub gain_db: f64,
    /// Things the pack uses that the package cannot express.
    pub not_converted: Vec<String>,
}

/// Convert the pack in `pack_dir` into a new package folder `out_dir`, which must
/// not exist. `out_dir` is the `altsound/<rom>/` folder the package will live in.
pub fn convert(
    pack_dir: &Path,
    out_dir: &Path,
    options: &Options,
) -> Result<Report, Box<dyn Error>> {
    if out_dir.exists() {
        return Err(format!("'{}' already exists", out_dir.display()).into());
    }
    let pack = Pack::load(pack_dir)?;
    let mut report = Report {
        not_converted: pack.extras.clone(),
        ..Default::default()
    };

    if let Some(target) = options.target_db {
        report.loudness_db = loudness::pack_mean_volume_db(
            pack.sounds
                .iter()
                .flat_map(|s| s.files.iter())
                .map(|p| p.as_path()),
        );
        if let Some(level) = report.loudness_db {
            report.gain_db = (target - level).max(0.0);
        }
    }

    let mut alt_rows = String::new();
    let mut gs_rows = String::new();
    let mut copies = Vec::new();
    let mut commands = std::collections::BTreeSet::new();
    for sound in &pack.sounds {
        // Command 0 is stop-all on every sound board; PinSound never plays a sound for it
        if sound.command == 0 {
            report.not_converted.push(format!(
                "{}/{}: command 0 is stop-all, never played",
                sound.kind, sound.folder
            ));
            continue;
        }
        if let Some(ms) = sound.delay {
            report
                .not_converted
                .push(format!("{}/{}: delay {ms}", sound.kind, sound.folder));
        }
        let name = sample_name(&sound.label);
        let volume = pack.volume(sound);
        let (channel, loop_, stop) = altsound_layout(&sound.kind);
        let duck = pack.ducking(&sound.kind).unwrap_or(100);
        let (gs_type, profile) = gsound_layout(&sound.kind);
        for (i, file) in sound.files.iter().enumerate() {
            let fname = sample_file_name(sound, &name, i, file);
            let _ = write!(
                alt_rows,
                "\"0x{:04x}\", {channel}, {duck}, {volume}, {loop_}, {stop}, \"{name}\", \"{fname}\"\r\n",
                sound.command
            );
            let _ = write!(
                gs_rows,
                "0x{:04x},{gs_type},{volume},{profile},{fname}\r\n",
                sound.command
            );
            copies.push((file.clone(), fname));
            report.samples += 1;
        }
        commands.insert(sound.command);
    }
    report.commands = commands.len();

    fs::create_dir_all(out_dir)?;
    for (src, dst) in &copies {
        fs::copy(src, out_dir.join(dst))?;
    }
    match options.format {
        Format::AltSound => {
            let header =
                "\"ID\",\"CHANNEL\",\"DUCK\",\"GAIN\",\"LOOP\",\"STOP\",\"NAME\",\"FNAME\"\r\n";
            fs::write(out_dir.join("altsound.csv"), header.to_string() + &alt_rows)?;
            fs::write(
                out_dir.join("altsound.ini"),
                crlf(&system_ini("altsound", report.gain_db)),
            )?;
        }
        Format::GSound => {
            let header = "ID,TYPE,GAIN,DUCKING_PROFILE,FNAME\r\n";
            fs::write(out_dir.join("g-sound.csv"), header.to_string() + &gs_rows)?;
            fs::write(
                out_dir.join("altsound.ini"),
                crlf(&gsound_ini(&pack, report.gain_db)),
            )?;
        }
    }
    Ok(report)
}

/// Channel, LOOP and STOP of a group in `altsound.csv`.
fn altsound_layout(kind: &Kind) -> (i32, u32, u32) {
    match kind {
        Kind::Music => (0, 100, 0),
        Kind::Jingle => (1, 0, 0),
        Kind::Single => (1, 0, 1),
        Kind::Sfx | Kind::Voice => (-1, 0, 0),
    }
}

/// G-Sound type and ducking profile of a group. The groups that duck use profile 1
/// of their type, defined in the ini.
fn gsound_layout(kind: &Kind) -> (&'static str, u32) {
    match kind {
        Kind::Music => ("MUSIC", 0),
        Kind::Single => ("SOLO", 0),
        Kind::Jingle => ("OVERLAY", 1),
        Kind::Voice => ("CALLOUT", 1),
        Kind::Sfx => ("SFX", 1),
    }
}

/// Label as a sample name: anything but letters, digits, `_`, `.` and `-` becomes `_`.
fn sample_name(label: &str) -> String {
    let name: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "_.-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    if name.is_empty() {
        "sound".to_string()
    } else {
        name
    }
}

/// `0x<command>-<name>[-<n>].<ext>`, flat in the package folder.
fn sample_file_name(sound: &Sound, name: &str, index: usize, file: &Path) -> String {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let suffix = if index == 0 {
        String::new()
    } else {
        format!("-{}", index + 1)
    };
    format!("0x{:04x}-{name}{suffix}.{ext}", sound.command)
}

fn system_ini(format: &str, gain_db: f64) -> String {
    format!(
        "[system]\nrecord_sound_cmds = 0\nrom_volume_ctrl = 1\ncmd_skip_count = 0\n\
         package_gain_db = {gain_db:.1}\n\n[format]\nformat = {format}\n\n\
         [logging]\nlogging_level = Error\n\n"
    )
}

/// G-Sound behaviour of a PinSound pack: music and single stop each other; jingle,
/// voice and sfx duck both to the pack's percentage (volume kept) and never stop or
/// pause anything.
fn gsound_ini(pack: &Pack, gain_db: f64) -> String {
    let ducker = |kind: Kind, extra: &str| {
        let t = gsound_layout(&kind).0.to_ascii_lowercase();
        let d = pack.ducking(&kind).unwrap_or(100);
        format!(
            "[{t}]\nducks = music, solo\n{extra}group_vol = 100\n\n\
             [{t}_ducking_profiles]\nducking_profile1 = music:{d}, solo:{d}\n\n"
        )
    };
    system_ini("g-sound", gain_db)
        + "[music]\nstops = solo\ngroup_vol = 100\n\n"
        + &ducker(Kind::Voice, "pauses =\nstops =\n")
        + &ducker(Kind::Sfx, "")
        + "[solo]\nstops = music\ngroup_vol = 100\n\n"
        + &ducker(Kind::Jingle, "")
}

fn crlf(s: &str) -> String {
    s.replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_names() {
        assert_eq!(
            sample_name("enter_your_name_high_score_"),
            "enter_your_name_high_score_"
        );
        assert_eq!(sample_name("game over!"), "game_over_");
        assert_eq!(sample_name(""), "sound");
    }

    #[test]
    fn gsound_ini_has_every_type() {
        let ini = gsound_ini(&Pack::default(), 12.3);
        assert!(ini.contains("package_gain_db = 12.3\n"));
        assert!(ini.contains("format = g-sound\n"));
        assert!(ini.contains("[overlay_ducking_profiles]\nducking_profile1 = music:5, solo:5\n"));
        assert!(ini.contains("[sfx_ducking_profiles]\nducking_profile1 = music:90, solo:90\n"));
        assert!(ini.contains("[callout_ducking_profiles]\nducking_profile1 = music:85, solo:85\n"));
    }
}
