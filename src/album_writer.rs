use std::fs;
use std::env;
use std::io::Result;

use crate::music_brainz::{Album};
use cd_da_reader::{Toc, CdReader};

pub fn write_album(album: &Album, reader: &CdReader, toc: &Toc) -> Result<()>  {
    let current_dir = env::current_dir()?;

    let new_dir= current_dir.join(&album.title);

    fs::create_dir(new_dir.as_path())?;

    for track in &album.tracks {
        let track_num = track.num.try_into().unwrap();
        let track_data = reader.read_track(toc, track_num)?;
        let wav_track = CdReader::create_wav(track_data);

        fs::write(new_dir.join(format!("{}.wav", &track.title)), wav_track)?;
    }

    Ok(())
}