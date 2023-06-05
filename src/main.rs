use clap::{arg, Command};
use psaperio::{config, scrambler::Scrambler, util};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

fn process_folder<P: AsRef<Path>>(scrambler: &Scrambler, path: &P) -> Result<(), std::io::Error> {
    if !path.as_ref().is_dir() {
        panic!("Path is not a directory");
    }

    for entry in read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        // We are only interested in directories
        if path.is_file() {
            continue;
        }

        let name = match path.file_name() {
            Some(name) => name.to_str().unwrap(),
            None => continue,
        };

        match util::split_id_str(name) {
            Some((id, description)) => {
                let num_id = id.parse::<u32>().unwrap();
                let new_id = match id.len() {
                    1..=5 => {
                        format!("{:010}", scrambler.scramble(num_id))
                    }
                    6..=10 => {
                        format!("{:05}", scrambler.unscramble(num_id))
                    }
                    _ => panic!("Invalid ID length"),
                };
                let new_name = format!("{}-{}", new_id, description);
                match std::fs::rename(&path, path.parent().unwrap().join(new_name)) {
                    Ok(_) => (),
                    Err(e) => println!(
                        "Failed to rename '{}' ({})",
                        path.join(name).to_str().unwrap(),
                        e
                    ),
                }
            }
            None => {
                println!("skipping: '{}'", name);
                continue;
            }
        };
    }
    Ok(())
}

const PS_CONFIG: &str = "config.pinsound";
const SOUND_FOLDERS: [&str; 5] = ["jingle", "music", "sfx", "single", "voice"];

fn process_soundpack(path: &Path) -> Result<(), std::io::Error> {
    let filename = match path.is_file() {
        true => path.to_path_buf(),
        false => path.join(PS_CONFIG),
    };

    let mut conf = config::PsConfig::new();
    let conf = conf.load(&filename)?;

    let scrambler = Scrambler::new(conf.get(config::CHECKSUM).unwrap());
    let directory = filename.parent().unwrap();
    for folder in SOUND_FOLDERS {
        let dir = directory.join(folder);
        if dir.exists() {
            println!("Processing '{}'", folder);
            process_folder(&scrambler, &dir)?;
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
                .arg(arg!(<FILE> "File to encrypt")),
        )
        .subcommand(
            Command::new("scramble")
                .about("Scramble a sound pack")
                .after_help("To use a custom config file place one inside sound pack folder and specify its path.\nIf target path is a folder the default config.pinsound file will be used.")
                .arg(
                    arg!(<PATH> "Path to folder or config file")
                    .value_parser(clap::value_parser!(PathBuf))))
        .subcommand(
            Command::new("unscramble")
                .about("Unscramble a sound pack")
                .after_help("To use a custom config file place one inside sound pack folder and specify its path.\nIf target path is a folder the default config.pinsound file will be used.")
                .arg(
                    arg!(<PATH> "Path to folder or config file")
                    .value_parser(clap::value_parser!(PathBuf)),
            ),
        )
}

fn main() {
    banner();
    let args = cli().get_matches();

    match args.subcommand() {
        Some(("decrypt", sub_args)) => {
            todo!()
        }
        Some(("encrypt", sub_args)) => {
            todo!()
        }
        Some(("scramble", sub_args)) => {
            todo!()
        }
        Some(("unscramble", sub_args)) => {
            let path = sub_args.get_one::<PathBuf>("PATH");
            if let Err(e) = process_soundpack(path.unwrap()) {
                println!("error: {}", e);
            }
        }
        _ => unreachable!(),
    }
}
