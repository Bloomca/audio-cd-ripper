use std::env;
use std::fs;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use ureq;

use crate::music_brainz::{Album, AlbumTrack};
use cd_da_reader::{CdReader, Toc};

use flac_codec::{
    byteorder::LittleEndian,
    encode::{FlacByteWriter, Options},
    metadata::{VorbisComment, update},
};

pub fn write_album(album: &Album, reader: &CdReader, toc: &Toc) -> Result<()> {
    let current_dir = env::current_dir()?;

    let new_dir = current_dir.join(&album.title);

    if new_dir.exists() {
        println!("Folder {} already exists, quitting", new_dir.display());
        return Ok(());
    }

    println!("Creating new folder for the album: {}", new_dir.display());

    fs::create_dir(new_dir.as_path())?;

    for track in &album.tracks {
        let Ok(track_num) = track.num.try_into() else {
            println!(
                "Could not convert the track number into u8 for {}. The value is {}.",
                &track.title, track.num
            );
            continue;
        };

        println!("Writing a track #{}: {}", track_num, &track.title);

        match reader.read_track(toc, track_num) {
            Ok(track_data) => {
                save_raw_data_as_flac(new_dir.join(&track.title), track_data, track, album)
            }
            Err(error) => {
                println!("Could not read track #{}, {}", track_num, &track.title);
                println!("Error: {:#?}", error);
                return Err(error);
            }
        };

        println!(
            "Successfully wrote the track #{}: {}",
            track_num, &track.title
        );
    }

    match fetch_album_art(album, &new_dir) {
        Ok(_) => {
            // pass, the success message is baked into the file
        }
        Err(error) => {
            println!(
                "Could not fetch cover art for {} by {}",
                &album.title, &album.artist
            );
            println!("{:#?}", error);
        }
    }

    println!("Successfully saved the album data");

    Ok(())
}

fn save_raw_data_as_flac(
    file_path: PathBuf,
    data: Vec<u8>,
    track: &AlbumTrack,
    album: &Album,
) -> Option<()> {
    let file = file_path.with_extension("flac");

    if file.exists() {
        println!("File {} already exists", file.display());
        return None;
    }

    // CD-DA: 44_100 Hz, 16-bit, 2 channels
    let sample_rate = 44_100u32;
    let bits_per_sample = 16u32;
    let channels = 2u8;

    {
        let mut flac_writer: FlacByteWriter<std::io::BufWriter<fs::File>, LittleEndian> =
            FlacByteWriter::create(
                &file,
                Options::best(),
                sample_rate,
                bits_per_sample,
                channels,
                None,
            )
            .unwrap();

        flac_writer.write_all(&data).ok()?;

        flac_writer.finalize().ok()?;
    }

    match update_track_metadata(&file, track, album) {
        Ok(_) => {
            println!("Successfully added metadata");
        }
        Err(flac_error) => {
            println!("{:#?}", flac_error);
        }
    };

    Some(())
}

fn update_track_metadata(
    file_path: &PathBuf,
    track: &AlbumTrack,
    album: &Album,
) -> std::result::Result<bool, flac_codec::Error> {
    update(file_path, |blocklist| {
        blocklist.update::<VorbisComment>(|vorbis_comment| {
            vorbis_comment.set("TITLE", &track.title);
            vorbis_comment.set("ALBUM", &album.title);
            vorbis_comment.set("ARTIST", &album.artist);
            vorbis_comment.set("TRACKNUMBER", track.num);
            vorbis_comment.set("DATE", &album.date);
            vorbis_comment.set("COUNTRY", &album.country);
        });

        Ok::<(), flac_codec::Error>(())
    })
}

fn fetch_album_art(
    album: &Album,
    directory_path: &PathBuf,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let Some(front_cover_url) = &album.front_cover_url else {
        return Ok(());
    };

    let user_agent = "audio-cd-ripper/0.1.0 (mail@bloomca.me)";
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .into();

    let response = agent
        .get(front_cover_url)
        .header("User-Agent", user_agent)
        .call()?;

    let content_type = response.headers().get("content-type");

    let ext = match content_type {
        Some(value) => match value.to_str().unwrap_or("image/jpeg") {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            _ => "jpg",
        },
        None => "jpg",
    };

    let mut bytes = Vec::new();
    response.into_body().into_reader().read_to_end(&mut bytes)?;

    let file_path = directory_path.join(format!("folder.{ext}"));
    let mut file = fs::File::create(&file_path)?;
    file.write_all(&bytes)?;

    println!("Cover art saved to: {}", file_path.display());
    Ok(())
}
