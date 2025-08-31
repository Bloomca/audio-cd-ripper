use base64::{Engine as _, engine::general_purpose};
use sha1::{Digest, Sha1};

use cd_da_reader::Toc;

pub fn calculate_music_brainz_id(toc: &Toc) -> String {
    let toc_string = format_toc_string(toc);

    let mut hasher = Sha1::new();
    hasher.update(toc_string.as_bytes());
    let hash_result = hasher.finalize();

    let base64_result = general_purpose::STANDARD.encode(&hash_result);

    // Convert to MusicBrainz format: replace + with . and / with _, remove padding
    base64_result
        .replace('+', ".")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

fn format_toc_string(toc: &Toc) -> String {
    let mut toc_string = String::new();

    // Add first track number (2 hex digits, uppercase)
    toc_string.push_str(&format!("{:02X}", toc.first_track));

    // Add last track number (2 hex digits, uppercase)
    toc_string.push_str(&format!("{:02X}", toc.last_track));

    // Add leadout offset (8 hex digits, uppercase)
    // MusicBrainz expects the leadout LBA + 150 (for the 2-second pregap)
    // audio CDs use 75 frames per second format.
    toc_string.push_str(&format!("{:08X}", toc.leadout_lba + 150));

    for track_num in 1..=99 {
        if let Some(track) = toc.tracks.iter().find(|t| t.number == track_num) {
            // Track exists: add its LBA offset + 150
            toc_string.push_str(&format!("{:08X}", track.start_lba + 150));
        } else {
            // Track doesn't exist: add 0
            toc_string.push_str("00000000");
        }
    }

    toc_string
}
