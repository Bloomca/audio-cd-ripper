use super::calculate_id;
use cd_da_reader::Toc;
use serde::Deserialize;
use std::time::Duration;

use ureq;

#[derive(Debug, Deserialize)]
pub struct MusicBrainzResponse {
    pub releases: Option<Vec<Release>>,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "release-group")]
    pub release_group: Option<ReleaseGroup>,
    #[serde(rename = "cover-art-archive")]
    pub cover_art_archive: Option<CoverArtArchive>,
    pub media: Option<Vec<Medium>>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<ArtistCredit>>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseGroup {
    pub id: String,
    pub title: Option<String>,
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CoverArtArchive {
    pub artwork: bool,
    pub count: u32,
    pub front: bool,
    pub back: bool,
}

#[derive(Debug, Deserialize)]
pub struct Medium {
    pub format: Option<String>, // "CD", etc.
    pub position: Option<u32>,
    #[serde(rename = "track-count")]
    pub track_count: Option<u32>,
    pub tracks: Option<Vec<Track>>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub id: String,
    pub number: Option<String>, // "1", "2", …
    pub title: Option<String>,
    pub length: Option<u32>, // ms
}

#[derive(Debug, Deserialize)]
pub struct ArtistCredit {
    pub name: String,               // credited name on this release/track
    pub joinphrase: Option<String>, // " & ", " feat. ", etc.
    pub artist: Option<Artist>,     // canonical artist entity
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub id: String,   // MBID
    pub name: String, // canonical name
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
}

#[derive(Debug)]
pub enum MusicBrainzError {
    Network(ureq::Error),
    Parse(serde_json::Error),
    NotFound,
    RateLimited,
}

impl From<ureq::Error> for MusicBrainzError {
    fn from(error: ureq::Error) -> Self {
        match &error {
            ureq::Error::StatusCode(404) => MusicBrainzError::NotFound,
            ureq::Error::StatusCode(429) => MusicBrainzError::RateLimited,
            _ => MusicBrainzError::Network(error),
        }
    }
}

impl From<serde_json::Error> for MusicBrainzError {
    fn from(error: serde_json::Error) -> Self {
        MusicBrainzError::Parse(error)
    }
}

pub struct MusicBrainzClient {
    agent: ureq::Agent,
    user_agent: String,
}

impl MusicBrainzClient {
    pub fn new(app_name: &str, app_version: &str, contact: &str) -> Self {
        let user_agent = format!("{app_name}/{app_version} ({contact})");
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();

        Self { agent, user_agent }
    }

    fn lookup_by_disc_id(
        &self,
        id: &str,
        includes: &[&str],
    ) -> Result<MusicBrainzResponse, MusicBrainzError> {
        let mut url: String = format!("https://musicbrainz.org/ws/2/discid/{}", id);

        if !includes.is_empty() {
            url.push_str(&format!("?inc={}", includes.join("+")));
        }

        let response = self
            .agent
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .call()?
            .body_mut()
            .read_json()?;

        Ok(response)
    }

    pub fn lookup_metadata(&self, toc: &Toc) -> Option<Album> {
        let id = calculate_id::calculate_music_brainz_id(toc);

        println!("MusicBrainzId: {id}");

        let includes: Vec<&str> = vec!["recordings", "artist-credits"];

        let result = self.lookup_by_disc_id(&id, &includes);

        match result {
            Ok(response) => {
                let album_data: Option<Album> = self.parse_metadata(&response);

                println!("{:#?}", album_data);

                album_data
            }
            Err(error) => {
                println!("{:#?}", error);

                None
            }
        }
    }

    fn parse_metadata(&self, response: &MusicBrainzResponse) -> Option<Album> {
        let Some(releases) = &response.releases else {
            return None;
        };

        if releases.is_empty() {
            return None;
        }

        let Some(release) = releases.get(0) else {
            return None;
        };

        let title = release.title.clone();
        let country = release.country.clone().unwrap_or("unknown".to_string());
        let date = release.date.clone().unwrap_or("Unknown date".to_string());

        let Some(cd_media) = self.find_cd_media(&release.media) else {
            return None;
        };

        let Some(tracks) = &cd_media.tracks else {
            return None;
        };

        let album_tracks = self.parse_album_tracks(tracks);

        let album = Album {
            title,
            country,
            tracks: album_tracks,
            artist: self.parse_artist(&release.artist_credit),
            date,
        };

        Some(album)
    }

    /// for now, find the first CD media and return it
    fn find_cd_media<'a>(&self, media: &'a Option<Vec<Medium>>) -> Option<&'a Medium> {
        let Some(mediums) = media else {
            return None;
        };

        for medium in mediums {
            if let Some(medium_format) = &medium.format {
                if medium_format == "CD" {
                    return Some(medium);
                }
            }
        }

        return None;
    }

    fn parse_album_tracks(&self, tracks: &Vec<Track>) -> Vec<AlbumTrack> {
        let mut result: Vec<AlbumTrack> = Vec::new();

        for track in tracks {
            if let Some(album_track) = AlbumTrack::new(track) {
                result.push(album_track);
            }
        }

        result
    }

    fn parse_artist(&self, artist_credit: &Option<Vec<ArtistCredit>>) -> String {
        let Some(artist_credit) = artist_credit else {
            return "Unknown artist".to_string();
        };

        if artist_credit.is_empty() {
            return "Unknown artist".to_string();
        }

        // TODO: add other credited artists
        let Some(artist) = artist_credit.get(0) else {
            return "Unknown artist".to_string();
        };

        return artist.name.clone();
    }
}

#[derive(Debug)]
pub struct Album {
    pub title: String,
    pub country: String,
    pub date: String,
    pub artist: String,
    pub tracks: Vec<AlbumTrack>,
}

#[derive(Debug)]
pub struct AlbumTrack {
    pub num: u32,
    pub title: String,
    pub len: u32,
}

impl AlbumTrack {
    fn new(track: &Track) -> Option<Self> {
        let Some(track_num) = &track.number else {
            return None;
        };

        let Ok(parsed_track_num) = track_num.parse::<u32>() else {
            return None;
        };

        let title = track.title.clone().unwrap_or("unknown track".to_string());

        let track_len = track.length.unwrap_or(0);

        Some(Self {
            num: parsed_track_num,
            title,
            len: track_len,
        })
    }
}
