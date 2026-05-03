use base64::{Engine as _, engine::general_purpose};
use sha1::{Digest, Sha1};

use cd_da_reader::{Toc, Track};

const CD_EXTRA_SESSION_GAP_FRAMES: u32 = 11_400;

pub struct MusicBrainzDisc {
    pub id: String,
    pub toc_query: String,
}

/// read more about the algorithm here: https://musicbrainz.org/doc/Disc_ID_Calculation
pub fn calculate_music_brainz_disc(toc: &Toc) -> MusicBrainzDisc {
    let toc_data = music_brainz_toc_data(toc);
    let toc_string = format_toc_string(&toc_data);

    let mut hasher = Sha1::new();
    hasher.update(toc_string.as_bytes());
    let hash_result = hasher.finalize();

    let base64_result = general_purpose::STANDARD.encode(hash_result);

    // Convert to MusicBrainz format: replace + with . and / with _, remove padding
    let id = base64_result
        .replace('+', ".")
        .replace('/', "_")
        .replace('=', "-")
        .to_string();

    MusicBrainzDisc {
        id,
        toc_query: format_toc_query(&toc_data),
    }
}

struct MusicBrainzTocData<'a> {
    first_track: u8,
    last_track: u8,
    leadout_lba: u32,
    tracks: Vec<&'a Track>,
}

fn music_brainz_toc_data(toc: &Toc) -> MusicBrainzTocData<'_> {
    let audio_tracks: Vec<&Track> = toc.tracks.iter().filter(|track| track.is_audio).collect();

    if audio_tracks.is_empty() {
        return MusicBrainzTocData {
            first_track: toc.first_track,
            last_track: toc.last_track,
            leadout_lba: toc.leadout_lba,
            tracks: toc.tracks.iter().collect(),
        };
    }

    let first_audio_track = audio_tracks.first().expect("audio_tracks is not empty");
    let last_audio_track = audio_tracks.last().expect("audio_tracks is not empty");
    let first_data_track_after_audio = toc
        .tracks
        .iter()
        .filter(|track| !track.is_audio && track.start_lba > last_audio_track.start_lba)
        .min_by_key(|track| track.start_lba);

    let leadout_lba = match first_data_track_after_audio {
        Some(data_track) => data_track
            .start_lba
            .saturating_sub(CD_EXTRA_SESSION_GAP_FRAMES),
        None => toc.leadout_lba,
    };

    MusicBrainzTocData {
        first_track: first_audio_track.number,
        last_track: last_audio_track.number,
        leadout_lba,
        tracks: audio_tracks,
    }
}

fn format_toc_string(toc: &MusicBrainzTocData<'_>) -> String {
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

fn format_toc_query(toc: &MusicBrainzTocData<'_>) -> String {
    let mut parts = Vec::with_capacity(toc.tracks.len() + 3);
    parts.push(toc.first_track.to_string());
    parts.push(toc.last_track.to_string());
    parts.push((toc.leadout_lba + 150).to_string());

    for track in &toc.tracks {
        parts.push((track.start_lba + 150).to_string());
    }

    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::{calculate_music_brainz_disc, calculate_music_brainz_id};
    use cd_da_reader::{Toc, Track};

    #[test]
    fn calculates_official_music_brainz_audio_cd_example() {
        let toc = toc(
            1,
            6,
            95_312,
            &[
                (1, 0, true),
                (2, 15_213, true),
                (3, 32_164, true),
                (4, 46_442, true),
                (5, 63_264, true),
                (6, 80_339, true),
            ],
        );

        assert_eq!(
            calculate_music_brainz_id(&toc),
            "49HHV7Eb8UKF3aQiNmu1GR8vKTY-"
        );
    }

    #[test]
    fn excludes_cd_extra_data_track_and_adjusts_audio_leadout() {
        let toc = toc(
            1,
            16,
            327_708,
            &[
                (1, 0, true),
                (2, 4_510, true),
                (3, 18_410, true),
                (4, 36_447, true),
                (5, 55_493, true),
                (6, 71_048, true),
                (7, 87_017, true),
                (8, 110_864, true),
                (9, 128_019, true),
                (10, 143_293, true),
                (11, 161_480, true),
                (12, 178_164, true),
                (13, 196_219, true),
                (14, 216_169, true),
                (15, 233_727, true),
                (16, 263_053, false),
            ],
        );

        let disc = calculate_music_brainz_disc(&toc);

        assert_eq!(disc.id, "c5wj9buAu_oxDzVSQzCo3ySEGxs-");
        assert_eq!(
            disc.toc_query,
            "1+15+251803+150+4660+18560+36597+55643+71198+87167+111014+128169+143443+161630+178314+196369+216319+233877"
        );
    }

    fn toc(first_track: u8, last_track: u8, leadout_lba: u32, tracks: &[(u8, u32, bool)]) -> Toc {
        Toc {
            first_track,
            last_track,
            leadout_lba,
            tracks: tracks
                .iter()
                .map(|(number, start_lba, is_audio)| Track {
                    number: *number,
                    start_lba: *start_lba,
                    start_msf: (0, 0, 0),
                    is_audio: *is_audio,
                })
                .collect(),
        }
    }
}
