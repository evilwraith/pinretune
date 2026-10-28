// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Wes Smith

//! Loudness of a sound pack.
//!
//! PinSound packs are mastered at very different levels: one pack can average
//! 12 dB below another. On PinSound hardware the board's amplifier makes up for
//! it; a converted package needs a level of its own (`package_gain_db`).
//!
//! The measure is the mean volume of each file (RMS over all samples, in dBFS),
//! as ffmpeg's `volumedetect` reports it, averaged over the pack's files.

use std::path::Path;

/// Mean volume of a WAV file in dBFS, or `None` if it cannot be read
/// (not a WAV file, unsupported sample format, or silent).
pub fn mean_volume_db(path: &Path) -> Option<f64> {
    let mut reader = hound::WavReader::open(path).ok()?;
    let spec = reader.spec();
    let (sum, count) = match spec.sample_format {
        hound::SampleFormat::Float => {
            sum_squares(reader.samples::<f32>().map(|s| s.map(f64::from)))
        }
        hound::SampleFormat::Int => {
            let full_scale = (1i64 << (spec.bits_per_sample - 1)) as f64;
            sum_squares(
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f64 / full_scale)),
            )
        }
    }?;
    if count == 0 || sum == 0.0 {
        return None;
    }
    Some(10.0 * (sum / count as f64).log10())
}

fn sum_squares<E>(samples: impl Iterator<Item = Result<f64, E>>) -> Option<(f64, u64)> {
    let mut sum = 0.0;
    let mut count = 0u64;
    for s in samples {
        let s = s.ok()?;
        sum += s * s;
        count += 1;
    }
    Some((sum, count))
}

/// Average mean volume over a set of files, skipping files that cannot be measured.
pub fn pack_mean_volume_db<'a>(files: impl Iterator<Item = &'a Path>) -> Option<f64> {
    let levels: Vec<f64> = files.filter_map(mean_volume_db).collect();
    if levels.is_empty() {
        None
    } else {
        Some(levels.iter().sum::<f64>() / levels.len() as f64)
    }
}
