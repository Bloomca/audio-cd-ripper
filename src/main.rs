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

    read_drive(&disk, args.tracks.as_deref());
}

struct CliArgs {
    disk: Option<String>,
    tracks: Option<Vec<u8>>,
}

fn parse_args() -> CliArgs {
    let mut disk: Option<String> = None;
    let mut tracks: Option<Vec<u8>> = None;
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

        if let Some(value) = arg.strip_prefix("--track=") {
            tracks = Some(parse_tracks(value));
            continue;
        }

        if arg == "--track" {
            let Some(value) = args.next() else {
                println!("Expected a value after --track");
                std::process::exit(2);
            };

            tracks = Some(parse_tracks(&value));
            continue;
        }
    }

    CliArgs { disk, tracks }
}

fn parse_tracks(value: &str) -> Vec<u8> {
    let mut tracks = Vec::new();

    for part in value.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(track) = trimmed.parse::<u8>() else {
            println!("Invalid track number in --track: {trimmed}");
            std::process::exit(2);
        };

        if track > 99 {
            println!("Track number out of range in --track: {track} (expected 0..=99)");
            std::process::exit(2);
        }

        tracks.push(track);
    }

    if tracks.is_empty() {
        println!("--track was provided but no valid track numbers were found");
        std::process::exit(2);
    }

    tracks.sort();
    tracks.dedup();
    tracks
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

fn read_drive(letter: &str, tracks: Option<&[u8]>) {
    let result = read_drive::read_drive(letter, tracks);

    if result.is_err() {
        println!("Error while reading drive {}", letter);
    } else {
        println!("Success!");
    }
}
