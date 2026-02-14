mod album_writer;
mod cli;
mod music_brainz;
mod read_drive;

use cd_da_reader::CdReader;

fn main() {
    let args = cli::parse_cli_args();

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

    read_drive(
        &disk,
        args.tracks.as_deref(),
        args.out_dir.as_path(),
        args.skip_artwork,
    );
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

fn read_drive(letter: &str, tracks: Option<&[u8]>, out_dir: &std::path::Path, skip_artwork: bool) {
    let result = read_drive::read_drive(letter, tracks, out_dir, skip_artwork);

    if result.is_err() {
        println!("Error while reading drive {}", letter);
    } else {
        println!("Success!");
    }
}
