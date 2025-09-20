use std::io::{Result, Error};
use crate::music_brainz::MusicBrainzClient;
use crate::album_writer::write_album;

use cd_da_reader::CdReader;

pub fn read_drive(letter: &str) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader.read_toc()?;

    let client = MusicBrainzClient::new("audio-cd-ripper", "0.1.0", "mail@bloomca.me");

    let Some(album) = client.lookup_metadata(&toc) else {
        return Err(Error::other("could not get album data"));
    };

    write_album(&album, &reader, &toc)?;

    Ok(())
}

