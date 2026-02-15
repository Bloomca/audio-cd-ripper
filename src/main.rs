mod album_writer;
mod cli;
mod music_brainz;
mod read_drive;

use cd_da_reader::CdReader;

fn main() {
    let args = cli::parse_cli_args();

    let (reader, drive_label) = match args.disk {
        Some(disk) => match CdReader::open(&disk) {
            Ok(reader) => (reader, disk),
            Err(error) => {
                println!("Could not open drive {}: {}", disk, error);
                std::process::exit(1);
            }
        },
        None => match CdReader::open_default() {
            Ok(reader) => (reader, "default drive".to_string()),
            Err(error) => {
                println!("Could not find any drive with an audio CD: {error}");
                std::process::exit(1);
            }
        },
    };

    read_drive(
        reader,
        &drive_label,
        args.tracks.as_deref(),
        args.out_dir.as_path(),
        args.skip_artwork,
    );
}

fn read_drive(
    reader: CdReader,
    drive_label: &str,
    tracks: Option<&[u8]>,
    out_dir: &std::path::Path,
    skip_artwork: bool,
) {
    let result = read_drive::read_drive(reader, drive_label, tracks, out_dir, skip_artwork);

    if result.is_err() {
        println!("Error while reading drive {}", drive_label);
    } else {
        println!("Success!");
    }
}
