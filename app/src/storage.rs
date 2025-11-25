use indexmap::IndexMap;
use rand::random_range;
use serde::{Deserialize, Serialize};

pub struct Storage {
    pub tracks: TracksJson
}
#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct TrackInfo {
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "ArtistID")]
    pub artist_id: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "msDuration")]
    pub ms_duration: u64,
    #[serde(rename = "TimesPlayed")]
    pub times_played: u32,
    #[serde(rename = "msPlayed")]
    pub ms_played: u64,
    #[serde(rename = "timeDistribution")]
    pub time_distribution: Vec<u32>,
    #[serde(rename = "Popularity")]
    pub popularity: u32,
}

/// The root of tracks.json
#[derive(Deserialize, Serialize)]
pub struct TracksJson {
    #[serde(flatten)]
    map: IndexMap<String, TrackInfo>,
}

impl TracksJson {
    pub fn get_track(&self, id: String) -> Option<&TrackInfo> {
        self.map.get(&id)
    }

    pub fn random_track(&self) -> Option<&TrackInfo> {
        let index = random_range(0..self.map.len());
        self.map.get_index(index).map(|(_, track)| track)
    }
}