use crate::spotify::{SpotifyClient, SpotifyError};
use crate::types::TrackInfo;
use indexmap::IndexMap;
use rand::random_range;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared multi-user storage: user name → their Storage
pub type UserStorages = Arc<RwLock<HashMap<String, Arc<RwLock<Storage>>>>>;

/// Number of tracks to keep pre-fetched and ready
const PREFETCH_SIZE: usize = 2;

pub struct Storage {
    pub tracks: TracksJson,
    prefetch_queue: RwLock<VecDeque<TrackInfo>>,
}

impl Storage {
    pub fn from_file<P: AsRef<Path>>(data_path: P, min_streams: u32) -> io::Result<Self> {
        let f = BufReader::new(std::fs::File::open(data_path)?);
        let tracks: TracksJson = serde_json::from_reader(f)?;
        let tracks = TracksJson::from_map(
            tracks
                .map
                .into_iter()
                .filter(|(_, track)| track.times_played >= min_streams)
                .collect(),
        );
        Ok(Storage {
            tracks,
            prefetch_queue: RwLock::new(VecDeque::new()),
        })
    }

    /// Create Storage from a TracksJson object
    pub fn from_tracks_json(tracks: TracksJson) -> Self {
        Storage {
            tracks,
            prefetch_queue: RwLock::new(VecDeque::new()),
        }
    }

    pub fn empty() -> Self {
        Storage {
            tracks: TracksJson {
                map: IndexMap::new(),
            },
            prefetch_queue: RwLock::new(VecDeque::new()),
        }
    }

    /// Pick a random PersonalTrackInfo from the track list (cheap, no I/O).
    pub fn pick_random_personal(&self) -> Option<PersonalTrackInfo> {
        if self.tracks.map.is_empty() {
            return None;
        }
        let index = random_range(0..self.tracks.map.len());
        self.tracks.map.get_index(index).map(|(_, t)| t.clone())
    }

    /// Pop one pre-fetched track from the queue, or None if the queue is empty.
    pub async fn pop_prefetched(&self) -> Option<TrackInfo> {
        self.prefetch_queue.write().await.pop_front()
    }

    /// Push a freshly-fetched track onto the pre-fetch queue.
    pub async fn push_prefetched(&self, track: TrackInfo) {
        self.prefetch_queue.write().await.push_back(track);
    }

    /// How many slots in the prefetch queue are currently unfilled.
    pub async fn prefetch_deficit(&self) -> usize {
        let len = self.prefetch_queue.read().await.len();
        PREFETCH_SIZE.saturating_sub(len)
    }

    // Kept for backward compat with the single-track route
    pub async fn random_track(
        &mut self,
        spotify_client: Arc<SpotifyClient>,
    ) -> Result<TrackInfo, SpotifyError> {
        if let Some(track) = self.prefetch_queue.write().await.pop_front() {
            return Ok(track);
        }
        // Queue empty — fetch synchronously
        loop {
            let Some(personal) = self.pick_random_personal() else {
                continue;
            };
            return TrackInfo::from_personal_track_info(personal, &spotify_client).await;
        }
    }

    // Kept for backward compat — fills one slot if not already full
    pub async fn gen_next_random(
        &mut self,
        spotify_client: Arc<SpotifyClient>,
    ) -> Result<(), SpotifyError> {
        if self.prefetch_queue.read().await.len() >= PREFETCH_SIZE {
            return Ok(());
        }
        loop {
            let Some(personal) = self.pick_random_personal() else {
                continue;
            };
            let track = TrackInfo::from_personal_track_info(personal, &spotify_client).await?;
            self.prefetch_queue.write().await.push_back(track);
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
    pub(crate) map: IndexMap<String, PersonalTrackInfo>,
}

impl TracksJson {
    /// Create a TracksJson from an IndexMap
    pub fn from_map(map: IndexMap<String, PersonalTrackInfo>) -> Self {
        TracksJson { map }
    }

    /// Get a reference to the internal map
    pub fn map(&self) -> &IndexMap<String, PersonalTrackInfo> {
        &self.map
    }
}
