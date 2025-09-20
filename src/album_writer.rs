use std::env;
use std::fs;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;

use crate::music_brainz::Album;
use cd_da_reader::{CdReader, Toc};

use flac_codec::{
    byteorder::LittleEndian,
    encode::{FlacByteWriter, Options},
};

pub fn write_album(album: &Album, reader: &CdReader, toc: &Toc) -> Result<()> {
    let current_dir = env::current_dir()?;

    let new_dir = current_dir.join(&album.title);

    fs::create_dir(new_dir.as_path())?;

    for track in &album.tracks {
        let track_num = track.num.try_into().unwrap();
        let track_data = reader.read_track(toc, track_num)?;
        save_raw_data_as_flac(new_dir.join(&track.title), track_data);
    }

    Ok(())
}

pub fn save_raw_data_as_flac(file_path: PathBuf, data: Vec<u8>) -> Option<()> {
    let file = file_path.with_extension("flac");

    if file.exists() {
        println!("File {} already exists", file.display());
        return None;
    }

    // CD-DA: 44_100 Hz, 16-bit, 2 channels
    let sample_rate = 44_100u32;
    let bits_per_sample = 16u32;
    let channels = 2u8;

    let total_samples_per_channel = data.len() as u64;

    let mut flac_writer: FlacByteWriter<std::io::BufWriter<fs::File>, LittleEndian> =
        FlacByteWriter::create(
            file,
            Options::default(),
            sample_rate,
            bits_per_sample,
            channels,
            Some(total_samples_per_channel),
        )
        .unwrap();

    flac_writer.write_all(&data).ok()?;

    Some(())
}
