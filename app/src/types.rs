use crate::spotify::{SpotifyClient, SpotifyError};
use crate::storage::PersonalTrackInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct TrackInfo {
    pub artist: String,
    pub artist_id: String,
    pub title: String,
    pub ms_duration: u64,
    pub times_played: u32,
    pub ms_played: u64,
    pub time_distribution: Vec<u32>,
    pub popularity: u32,

    pub spotify_url: Option<String>,
    pub preview_url: Option<String>,
    pub album_image_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct TwoTracksResponse {
    pub track1: TrackInfo,
    pub track2: TrackInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct AlbumImageResponse {
    pub track_id: String,
    pub album_image_url: Option<String>,
}

impl TrackInfo {
    pub async fn from_personal_track_info(
        personal: PersonalTrackInfo,
        spotify_client: &SpotifyClient,
    ) -> Result<Self, SpotifyError> {
        let spotify_track = spotify_client.get_track(personal.id.as_str()).await;

        Ok(TrackInfo {
            artist: personal.artist,
            artist_id: personal.artist_id,
            title: personal.title,
            ms_duration: personal.ms_duration,
            times_played: personal.times_played,
            ms_played: personal.ms_played,
            time_distribution: personal.time_distribution,
            popularity: personal.popularity,
            spotify_url: spotify_track.as_ref()
                .ok()
                .map(|track| track.external_urls.spotify.clone()),
            album_image_url: spotify_track.as_ref()
                .ok()
                .map(|track| track.album.images.first().map(|img| img.url.clone()))
                .flatten(),
            preview_url: spotify_track.as_ref()
                .ok()
                .map(|track| track.preview_url.clone())
                .flatten(),
        })
    }
}
