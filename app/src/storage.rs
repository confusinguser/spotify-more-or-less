use crate::spotify::{SpotifyClient, SpotifyError};
use crate::types::TrackInfo;
use indexmap::IndexMap;
use rand::random_range;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;
use tokio::sync::RwLock;

pub struct Storage {
    pub tracks: TracksJson,
    next_random: RwLock<Option<TrackInfo>>,
}

impl Storage {
}

impl Storage {
    pub(crate) fn from_file<P: AsRef<Path>>(data_path: P) -> io::Result<Self> {
        let f = std::fs::File::open(data_path)?;
        let tracks: TracksJson = serde_json::from_reader(f)?;
        Ok(Storage {
            tracks,
            next_random: RwLock::new(None),
        })
    }

    pub fn empty() -> Self {
        Storage {
            tracks: TracksJson {
                map: IndexMap::new(),
            },
            next_random: RwLock::new(None),
        }
    }

    pub(crate) fn sample_data() -> Self {
        let mut map = IndexMap::new();
        map.insert(
            "3n3Ppam7vgaVa1iaRUc9Lp".to_string(),
            PersonalTrackInfo {
                id: "3n3Ppam7vgaVa1iaRUc9Lp".to_string(),
                artist: "Red Hot Chili Peppers".to_string(),
                artist_id: "0L8ExT028jH3ddEcZwqJJ5".to_string(),
                title: "Californication".to_string(),
                ms_duration: 329000,
                times_played: 42,
                ms_played: 12345000,
                time_distribution: vec![0; 24],
                popularity: 85,
            },
        );
        map.insert(
            "4GZ3YCkuH0VvTluVLwUp4g".to_string(),
            PersonalTrackInfo {
                id: "4GZ3YCkuH0VvTluVLwUp4g".to_string(),
                artist: "I just wanna shine".to_string(),
                artist_id: "0L8ExT028jH3ddEcZwqJJ5".to_string(),
                title: "Californication".to_string(),
                ms_duration: 329000,
                times_played: 42,
                ms_played: 12345000,
                time_distribution: vec![0; 24],
                popularity: 85,
            },
        );

        Storage {
            tracks: TracksJson { map },
            next_random: RwLock::new(None),
        }

    }

    pub async fn random_track(&mut self, spotify_client: &SpotifyClient) -> Result<TrackInfo, SpotifyError> {
        if let Some(track) = self.next_random.write().await.take() {
            return Ok(track);
        }
        self.gen_next_random(spotify_client).await?;
        Ok(self.next_random.write().await.take().unwrap())
    }

    pub async fn gen_next_random(&mut self, spotify_client: &SpotifyClient) -> Result<(), SpotifyError> {
        if self.next_random.read().await.is_some() {
            return Ok(());
        }
        loop {
            let index = random_range(0..self.tracks.map.len());
            let Some(personal_track_info) = self.tracks.map.get_index(index).map(|(_, track)| {
                let mut track = track.clone();
                track.id = track.id.clone();
                track
            }) else {
                continue
            };
            let track_info = TrackInfo::from_personal_track_info(personal_track_info, &spotify_client)
                .await?;
            let mut next_random = self.next_random.write().await;
            *next_random = Some(track_info);
            break;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct PersonalTrackInfo {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: String,
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
    map: IndexMap<String, PersonalTrackInfo>,
}
