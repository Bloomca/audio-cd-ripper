use std::io::{Result};
use crate::music_brainz::calculate_music_brainz_id;

use cd_da_reader::CdReader;

pub fn read_drive(letter: &str) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader.read_toc()?;

    let id = calculate_music_brainz_id(&toc);

    println!("Music Brainz ID: {}", id);

    Ok(())
}

