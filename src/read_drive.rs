use crate::album_writer::write_album;
use crate::music_brainz::{Album, MusicBrainzClient};
use std::io::{Error, Result};
use std::path::Path;

use cd_da_reader::CdReader;

pub fn read_drive(
    letter: &str,
    tracks: Option<&[u8]>,
    out_dir: &Path,
    skip_artwork: bool,
) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader
        .read_toc()
        .map_err(|error| Error::other(error.to_string()))?;

    let client = MusicBrainzClient::new("audio-cd-ripper", "0.1.0", "mail@bloomca.me");

    let Some(album) = client.lookup_metadata(&toc) else {
        return Err(Error::other("could not get album data"));
    };

    print_album_info(&album);

    if let Some(selected_tracks) = tracks {
        println!("Only ripping selected tracks: {selected_tracks:?}");
    }

    if skip_artwork {
        println!("Skipping artwork as requested");
    }

    write_album(&album, &reader, &toc, tracks, out_dir, skip_artwork)?;

    Ok(())
}

fn print_album_info(album: &Album) {
    println!(
        "Found album {} by {} from {}, {} release",
        album.title, album.artist, album.date, album.country
    );
    println!("There are {} tracks total", album.tracks.len());
}
