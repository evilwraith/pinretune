// Copyright 2026 Martin Åkesson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing
// permissions and limitations under the License.

use clap::{Command, arg};
use psaperio::{file as psfile, scrambler::Scrambler, util};
use psconfig::parse;
use psconvert::altsound::{self, Format, Options};
use psconvert::pack::Pack;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const PS_CONFIG: &str = "config.pinsound";
const SOUND_FOLDERS: [&str; 5] = ["jingle", "music", "sfx", "single", "voice"];

/// Resolve to the config file path: use as-is if it points to a file,
/// otherwise append the default config filename.
fn resolve_config_path(path: &Path) -> PathBuf {
    if path.is_file() {
        path.to_path_buf()
    } else {
        path.join(PS_CONFIG)
    }
}

/// Rename sound folders by scrambling or unscrambling IDs.
fn process_folder(
    scrambler: &Scrambler,
    path: &Path,
    scramble: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.is_dir() {
        return Err(format!("'{}' is not a directory", path.display()).into());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        // Only process directories (sound id folders)
        if entry_path.is_file() {
            continue;
        }

        let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some((id_str, description)) = util::split_id_str(name) else {
            eprintln!("skipping: '{name}'");
            continue;
        };

        let Ok(num_id) = id_str.parse::<u32>() else {
            eprintln!("warning: invalid id '{id_str}' in '{name}'");
            continue;
        };

        let new_id = if scramble {
            if id_str.len() > 5 {
                eprintln!("warning: '{name}' appears already scrambled, skipping");
                continue;
            }
            format!("{:010}", scrambler.scramble(num_id))
        } else {
            if id_str.len() <= 5 {
                eprintln!("warning: '{name}' does not appear scrambled, skipping");
                continue;
            }
            format!("{:05}", scrambler.unscramble(num_id))
        };

        let new_name = format!("{new_id}-{description}");
        let new_path = entry_path.parent().unwrap().join(&new_name);
        if let Err(e) = fs::rename(&entry_path, &new_path) {
            eprintln!("warning: failed to rename '{name}': {e}");
        }
    }
    Ok(())
}

/// Load and parse the configuration, then scramble/unscramble all sound folders.
fn process_soundpack(path: &Path, scramble: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = resolve_config_path(path);
    let data = psfile::load(&config_path)?;
    let text = String::from_utf8_lossy(&data);
    let config = parse::parse(&text)?;

    let checksum = config
        .checksum()
        .ok_or("no checksum found in configuration file")?;
    let scrambler = Scrambler::new(checksum);

    let directory = config_path
        .parent()
        .ok_or("cannot determine parent directory")?;

    for folder in SOUND_FOLDERS {
        let dir = directory.join(folder);
        if dir.exists() {
            println!("Processing '{folder}'");
            process_folder(&scrambler, &dir, scramble)?;
        }
    }
    Ok(())
}

/// Print every sound of a pack with its unscrambled command ID.
fn list_soundpack(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pack = Pack::load(path)?;
    if let Some(roms) = pack.config.roms() {
        println!("roms: {}", roms.join(" "));
    }
    for sound in &pack.sounds {
        println!(
            "0x{:04x}\t{}\t{}\t{} file(s)",
            sound.command,
            sound.kind,
            sound.folder,
            sound.files.len()
        );
    }
    for extra in &pack.extras {
        println!("also uses: {extra}");
    }
    Ok(())
}

/// Convert a pack into an AltSound package.
fn convert_soundpack(
    path: &Path,
    out: &Path,
    options: &Options,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = altsound::convert(path, out, options)?;
    if let Some(level) = report.loudness_db {
        println!(
            "Pack loudness {level:.1} dB average, package_gain_db {:.1}",
            report.gain_db
        );
    }
    println!(
        "{} samples, {} commands -> '{}'",
        report.samples,
        report.commands,
        out.display()
    );
    let pack = Pack::load(path)?;
    match pack.config.roms() {
        Some(roms) => println!(
            "Pack ROMs: {} (name the folder after the ROM PinMAME runs)",
            roms.join(" ")
        ),
        None => println!("The pack lists no ROMs: name the folder after the ROM PinMAME runs"),
    }
    for item in &report.not_converted {
        println!("not converted: {item}");
    }
    Ok(())
}

fn banner() {
    println!(" _____________       ________     _____                    ");
    println!(" ___  __ \\__(_)_________  __ \\______  /____  _____________ ");
    println!(" __  /_/ /_  /__  __ \\_  /_/ /  _ \\  __/  / / /_  __ \\  _ \\");
    println!(" _  ____/_  / _  / / /  _, _//  __/ /_ / /_/ /_  / / /  __/");
    println!(" /_/     /_/  /_/ /_//_/ |_| \\___/\\__/ \\__,_/ /_/ /_/\\___/");
    println!("\n        --==[ ReTune your pinball sounds ]==--\n");
}

fn cli() -> Command {
    Command::new("pinretune")
        .about("Sound pack manipulation tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("decrypt")
                .about("Decrypt a configuration file in place")
                .arg(arg!(<FILE> "Encrypted file to decrypt in place")),
        )
        .subcommand(
            Command::new("encrypt")
                .about("Encrypt a configuration file in place")
                .arg(arg!(<FILE> "Plaintext file to encrypt in place")),
        )
        .subcommand(
            Command::new("scramble")
                .about("Scramble sound folder IDs in a sound pack")
                .after_help(
                    "To use a custom config file place one inside the sound pack folder and specify its path.\n\
                     If target path is a folder the default config.pinsound file will be used.",
                )
                .arg(
                    arg!(<PATH> "Path to folder or config file")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("unscramble")
                .about("Unscramble sound folder IDs in a sound pack")
                .after_help(
                    "To use a custom config file place one inside the sound pack folder and specify its path.\n\
                     If target path is a folder the default config.pinsound file will be used.",
                )
                .arg(
                    arg!(<PATH> "Path to folder or config file")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List the sounds of a sound pack with their command IDs")
                .arg(
                    arg!(<PATH> "Sound pack folder")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("convert")
                .about("Convert a sound pack into an AltSound package for PinMAME / VPX")
                .after_help(
                    "OUT is the package folder to create, altsound/<rom>/ in PinMAME's folder; it must not exist.\n\
                     The pack is raised to --target-db average loudness through package_gain_db in\n\
                     altsound.ini (never lowered); the audio is copied unchanged.",
                )
                .arg(
                    arg!(<PATH> "Sound pack folder")
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    arg!(<OUT> "Package folder to create")
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    arg!(--format <FORMAT> "Package format")
                        .value_parser(["altsound", "gsound"])
                        .default_value("gsound"),
                )
                .arg(
                    arg!(--"target-db" <DB> "Average loudness to raise the pack to, in dBFS")
                        .value_parser(clap::value_parser!(f64))
                        .allow_negative_numbers(true)
                        .default_value("-19"),
                )
                .arg(arg!(--"no-normalize" "Leave the pack's level as it is")),
        )
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    banner();
    let args = cli().get_matches();

    match args.subcommand() {
        Some(("decrypt", sub_args)) => {
            let file = sub_args.get_one::<String>("FILE").unwrap();
            let data = psfile::load(file)?;
            fs::write(file, &data)?;
            println!("Decrypted '{file}'");
        }
        Some(("encrypt", sub_args)) => {
            let file = sub_args.get_one::<String>("FILE").unwrap();
            let data = fs::read(file)?;
            psfile::save(file, &data)?;
            println!("Encrypted '{file}'");
        }
        Some(("scramble", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            process_soundpack(path, true)?;
        }
        Some(("unscramble", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            process_soundpack(path, false)?;
        }
        Some(("list", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            list_soundpack(path)?;
        }
        Some(("convert", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            let out = sub_args.get_one::<PathBuf>("OUT").unwrap();
            let format = match sub_args.get_one::<String>("format").unwrap().as_str() {
                "altsound" => Format::AltSound,
                _ => Format::GSound,
            };
            let target_db = if sub_args.get_flag("no-normalize") {
                None
            } else {
                sub_args.get_one::<f64>("target-db").copied()
            };
            convert_soundpack(path, out, &Options { format, target_db })?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
