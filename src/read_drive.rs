use std::io::{Result};

use cd_da_reader::CdReader;

pub fn read_drive(letter: &str) -> Result<()> {
    println!("Drive with a CD: {letter}");

    let reader = CdReader::open(letter)?;
    let toc = reader.read_toc()?;

    Ok(())
}

