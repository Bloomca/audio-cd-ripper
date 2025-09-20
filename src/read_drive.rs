use crate::album_writer::write_album;
use crate::music_brainz::{Album, MusicBrainzClient};
use std::io::{Error, Result};

use cd_da_reader::CdReader;

pub fn read_drive(letter: &str) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader.read_toc()?;

    let client = MusicBrainzClient::new("audio-cd-ripper", "0.1.0", "mail@bloomca.me");

    let Some(album) = client.lookup_metadata(&toc) else {
        return Err(Error::other("could not get album data"));
    };

    print_album_info(&album);

    write_album(&album, &reader, &toc)?;

    Ok(())
}

fn print_album_info(album: &Album) {
    println!(
        "Found album {} by {} from {}, {} release",
        album.title, album.artist, album.date, album.country
    );
    println!("There are {} tracks total", album.tracks.len());
}
