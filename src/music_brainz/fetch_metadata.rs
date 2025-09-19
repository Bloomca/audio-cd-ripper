use serde::Deserialize;
use std::time::Duration;
use cd_da_reader::Toc;
use super::calculate_id;

use ureq;

#[derive(Debug, Deserialize)]
pub struct MusicBrainzResponse {
    pub releases: Option<Vec<Release>>
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug)]
pub enum MusicBrainzError {
    Network(ureq::Error),
    Parse(serde_json::Error),
    NotFound,
    RateLimited
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
        let mut url = format!("https://musicbrainz.org/ws/2/discid/{}", id);

        if !includes.is_empty() {
            url.push_str(&format!("&inc={}", includes.join("+")));
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

    pub fn lookup_metadata(&self, toc: &Toc) {
        let id = calculate_id::calculate_music_brainz_id(toc);

        println!("MusicBrainzId: {id}");

        let includes: Vec<&str> = vec![];

        let result = self.lookup_by_disc_id(&id, &includes);

        println!("{:#?}", result);
    }
}
