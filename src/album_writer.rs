use std::fs;
use std::io::Read;
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::music_brainz::{Album, AlbumTrack};
use cd_da_reader::{CdReader, Toc, TrackStreamConfig};

use flac_codec::{
    byteorder::LittleEndian,
    encode::{FlacByteWriter, Options},
    metadata::{VorbisComment, update},
};

pub fn write_album(
    album: &Album,
    reader: &CdReader,
    toc: &Toc,
    selected_tracks: Option<&[u8]>,
    out_dir: &Path,
    skip_artwork: bool,
) -> Result<()> {
    let new_dir = out_dir.join(sanitize_title(&album.title));
    let mut all_requested_tracks: Vec<(u8, &AlbumTrack)> = Vec::new();

    for track in &album.tracks {
        let Ok(track_num) = track.num.try_into() else {
            println!(
                "Could not convert the track number into u8 for {}. The value is {}.",
                &track.title, track.num
            );
            continue;
        };

        if let Some(selected_tracks) = selected_tracks
            && !selected_tracks.contains(&track_num)
        {
            continue;
        }

        all_requested_tracks.push((track_num, track));
    }

    if new_dir.exists() {
        println!(
            "Folder {} already exists, checking for missing files",
            new_dir.display()
        );
    } else {
        println!("Creating new folder for the album: {}", new_dir.display());
        fs::create_dir(new_dir.as_path())?;
    }

    let mut tracks_to_rip: Vec<(u8, &AlbumTrack)> = Vec::new();
    for (track_num, track) in &all_requested_tracks {
        let track_path = track_file_path(&new_dir, track);
        if !track_path.exists() {
            tracks_to_rip.push((*track_num, *track));
        }
    }

    let artwork_missing = !has_album_art(&new_dir);
    let should_fetch_artwork = !skip_artwork && artwork_missing;
    if tracks_to_rip.is_empty() && !should_fetch_artwork {
        if skip_artwork {
            println!(
                "All requested tracks already exist in {}, nothing else to do",
                new_dir.display()
            );
        } else {
            println!(
                "All requested tracks and artwork already exist in {}, nothing else to do",
                new_dir.display()
            );
        }
        return Ok(());
    }

    if !tracks_to_rip.is_empty() {
        println!("Missing {} track(s), starting rip", tracks_to_rip.len());
    } else {
        println!("All requested tracks already exist, only artwork is missing");
    }

    if skip_artwork {
        println!("Skipping artwork download as requested");
    } else if !artwork_missing {
        println!("Artwork already exists, skipping cover download");
    }

    let mut failed_tracks: Vec<(u8, String)> = Vec::new();

    for (track_num, track) in tracks_to_rip {
        println!("\nWriting track #{}: {}", track_num, &track.title);

        match write_track_as_flac_with_progress(
            new_dir.join(sanitize_title(&track.title)),
            reader,
            toc,
            track_num,
            track,
            album,
        ) {
            Ok(_) => {}
            Err(error) => {
                println!("Could not write track #{}, {}", track_num, &track.title);
                println!("Error: {:#?}", error);
                failed_tracks.push((track_num, track.title.clone()));
                continue;
            }
        };

        println!(
            "\r  Successfully wrote the track #{}: {}",
            track_num, &track.title
        );
    }

    if failed_tracks.is_empty() {
        println!("All tracks were written successfully");
    } else {
        println!("Failed to write {} track(s):", failed_tracks.len());
        for (track_num, track_title) in &failed_tracks {
            println!("#{} {}", track_num, track_title);
        }
    }

    if should_fetch_artwork {
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
    };

    if failed_tracks.is_empty() {
        println!("Successfully saved the album data");
    } else {
        println!("Successfully saved some album data");
    }

    Ok(())
}

fn write_track_as_flac_with_progress(
    file_path: PathBuf,
    reader: &CdReader,
    toc: &Toc,
    track_num: u8,
    track: &AlbumTrack,
    album: &Album,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let file = file_path.with_extension("flac");

    if file.exists() {
        println!("File {} already exists", file.display());
        return Ok(());
    }

    let temp_file = file.with_extension("flac.tmp");
    if temp_file.exists() {
        fs::remove_file(&temp_file)?;
    }

    let result = (|| -> std::result::Result<(), Box<dyn std::error::Error>> {
        write_track_to_temp_flac(&temp_file, reader, toc, track_num)?;
        update_track_metadata(&temp_file, track, album)?;
        println!("\r  Successfully added metadata");
        fs::rename(&temp_file, &file)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_file);
    }

    result
}

fn write_track_to_temp_flac(
    file: &Path,
    reader: &CdReader,
    toc: &Toc,
    track_num: u8,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    const CDDA_SECTOR_BYTES: usize = 2352;

    let mut stream = reader.open_track_stream(toc, track_num, TrackStreamConfig::default())?;
    let total_sectors = stream.total_sectors();
    let total_secs = stream.total_seconds();
    let total_bytes = u64::from(total_sectors) * CDDA_SECTOR_BYTES as u64;
    let mut printed_progress = false;

    // CD-DA: 44_100 Hz, 16-bit, 2 channels
    let sample_rate = 44_100u32;
    let bits_per_sample = 16u32;
    let channels = 2u8;
    let total_bytes = if total_bytes == 0 {
        None
    } else {
        Some(total_bytes)
    };

    let mut flac_writer: FlacByteWriter<std::io::BufWriter<fs::File>, LittleEndian> =
        FlacByteWriter::create(
            file,
            Options::best(),
            sample_rate,
            bits_per_sample,
            channels,
            total_bytes,
        )?;

    loop {
        match stream.next_chunk() {
            Ok(Some(chunk)) => {
                if let Err(error) = flac_writer.write_all(&chunk) {
                    if printed_progress {
                        eprintln!();
                    }

                    return Err(error.into());
                }

                print_rip_progress(stream.current_seconds(), total_secs);
                printed_progress = true;
            }
            Ok(None) => break,
            Err(error) => {
                if printed_progress {
                    eprintln!();
                }

                return Err(error.into());
            }
        }
    }

    print_rip_progress(total_secs, total_secs);

    if let Err(error) = flac_writer.finalize() {
        eprintln!();
        return Err(error.into());
    }
    eprintln!();

    Ok(())
}

fn print_rip_progress(current_secs: f32, total_secs: f32) {
    let percent = if total_secs > 0.0 {
        current_secs / total_secs * 100.0
    } else {
        100.0
    };

    eprint!(
        "\r  Reading and encoding: [{} / {}] {:5.1}%",
        format_duration(current_secs),
        format_duration(total_secs),
        percent.min(100.0)
    );
}

fn format_duration(seconds: f32) -> String {
    let total_seconds = seconds.round() as u32;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    format!("{minutes:02}:{seconds:02}")
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
            if !album.genres.is_empty() {
                vorbis_comment.set("GENRE", album.genres.join(", "));
            }
        });

        Ok::<(), flac_codec::Error>(())
    })
}

fn fetch_album_art(
    album: &Album,
    directory_path: &Path,
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

fn has_album_art(directory_path: &Path) -> bool {
    directory_path.join("folder.jpg").exists()
        || directory_path.join("folder.jpeg").exists()
        || directory_path.join("folder.png").exists()
}

fn track_file_path(directory_path: &Path, track: &AlbumTrack) -> PathBuf {
    directory_path
        .join(sanitize_title(&track.title))
        .with_extension("flac")
}

fn sanitize_title(title: &str) -> String {
    // at least Windows prohibits these characters
    const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    title.chars().filter(|c| !FORBIDDEN.contains(c)).collect()
}
