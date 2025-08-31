use std::io::{Result};
use crate::music_brainz::MusicBrainzClient;

use cd_da_reader::CdReader;

pub fn read_drive(letter: &str) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader.read_toc()?;

    let client = MusicBrainzClient::new("audio-cd-ripper", "0.1.0", "mail@bloomca.me");

    client.lookup_metadata(&toc);

    Ok(())
}

