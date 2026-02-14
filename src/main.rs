mod album_writer;
mod music_brainz;
mod read_drive;

use cd_da_reader::CdReader;
use std::env;

fn main() {
    let args = parse_args();

    let disk = match args.disk {
        Some(disk) => disk,
        None => match detect_disk() {
            Some(disk) => disk,
            None => {
                println!("Could not find any drive with an audio CD");
                std::process::exit(1);
            }
        },
    };

    read_drive(&disk);
}

struct CliArgs {
    disk: Option<String>,
}

fn parse_args() -> CliArgs {
    let mut disk: Option<String> = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--disk=") {
            disk = Some(value.to_string());
            continue;
        }

        if arg == "--disk" {
            let Some(value) = args.next() else {
                println!("Expected a value after --disk");
                std::process::exit(2);
            };

            disk = Some(value);
            continue;
        }
    }

    CliArgs { disk }
}

fn detect_disk() -> Option<String> {
    let drives = match CdReader::list_drives() {
        Ok(drives) => drives,
        Err(error) => {
            println!("Failed to list CD drives: {error}");
            return None;
        }
    };

    let audio_drives: Vec<String> = drives
        .into_iter()
        .filter(|drive| drive.has_audio_cd)
        .map(|drive| drive.path)
        .collect();

    if audio_drives.is_empty() {
        return None;
    }

    if audio_drives.len() > 1 {
        println!(
            "Found multiple drives with an audio CD: {:#?}",
            audio_drives
        );
        println!("Trying to use the first one");
    }

    audio_drives.into_iter().next()
}

fn read_drive(letter: &str) {
    let result = read_drive::read_drive(letter);

    if result.is_err() {
        println!("Error while reading drive {}", letter);
    } else {
        println!("Success!");
    }
}
