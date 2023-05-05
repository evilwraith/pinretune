use psaperio::{config, scrambler::Scrambler, util};
use std::fs::read_dir;

pub fn process_folder(checksum: &str, path: &str) -> Result<(), std::io::Error> {
    let scrambler = Scrambler::new(checksum);

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

fn main() {
    let config = config::PsConfig::new();
    _ = config.load("config.pinsound");
    _ = process_folder("PantsOnFire", "TestPack/sfx");
}
