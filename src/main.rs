use clap::{Command, arg};
use psaperio::{file as psfile, scrambler::Scrambler, util};
use psconfig::parse;
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

        let name = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let (id_str, description) = match util::split_id_str(name) {
            Some(pair) => pair,
            None => {
                eprintln!("skipping: '{}'", name);
                continue;
            }
        };

        let num_id: u32 = match id_str.parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("warning: invalid id '{}' in '{}'", id_str, name);
                continue;
            }
        };

        let new_id = if scramble {
            if id_str.len() > 5 {
                eprintln!("warning: '{}' appears already scrambled, skipping", name);
                continue;
            }
            format!("{:010}", scrambler.scramble(num_id))
        } else {
            if id_str.len() <= 5 {
                eprintln!("warning: '{}' does not appear scrambled, skipping", name);
                continue;
            }
            format!("{:05}", scrambler.unscramble(num_id))
        };

        let new_name = format!("{}-{}", new_id, description);
        let new_path = entry_path.parent().unwrap().join(&new_name);
        if let Err(e) = fs::rename(&entry_path, &new_path) {
            eprintln!("warning: failed to rename '{}': {}", name, e);
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
            println!("Processing '{}'", folder);
            process_folder(&scrambler, &dir, scramble)?;
        }
    }
    Ok(())
}

fn banner() {
    println!(" _____________       ________     _____                    ");
    println!(" ___  __ \\__(_)_________  __ \\______  /____  _____________ ");
    println!(" __  /_/ /_  /__  __ \\_  /_/ /  _ \\  __/  / / /_  __ \\  _ \\");
    println!(" _  ____/_  / _  / / /  _, _//  __/ /_ / /_/ /_  / / /  __/");
    println!(" /_/     /_/  /_/ /_//_/ |_| \\___/\\__/ \\__,_/ /_/ /_/\\___/");
    println!("\n        --==[ Liberate your pinball sound ]==--\n");
}

fn cli() -> Command {
    Command::new("pinretune")
        .about("Sound pack manipulation tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("decrypt")
                .about("Decrypt a configuration file")
                .arg(arg!(<FILE> "File to decrypt")),
        )
        .subcommand(
            Command::new("encrypt")
                .about("Encrypt a configuration file")
                .arg(arg!(<INPUT> "Plaintext input file"))
                .arg(arg!(<OUTPUT> "Encrypted output file")),
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
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    banner();
    let args = cli().get_matches();

    match args.subcommand() {
        Some(("decrypt", sub_args)) => {
            let file = sub_args.get_one::<String>("FILE").unwrap();
            let data = psfile::load(file)?;
            let text = String::from_utf8_lossy(&data);
            println!("{}", text);
        }
        Some(("encrypt", sub_args)) => {
            let input = sub_args.get_one::<String>("INPUT").unwrap();
            let output = sub_args.get_one::<String>("OUTPUT").unwrap();
            let data = fs::read(input)?;
            psfile::save(output, &data)?;
            println!("Encrypted '{}' -> '{}'", input, output);
        }
        Some(("scramble", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            process_soundpack(path, true)?;
        }
        Some(("unscramble", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH").unwrap();
            process_soundpack(path, false)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
