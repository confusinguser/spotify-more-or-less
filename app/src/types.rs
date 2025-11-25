use serde::{Deserialize, Serialize};
use crate::spotify::{SpotifyClient, SpotifyError};
use crate::storage::PersonalTrackInfo;

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
    
    pub spotify_url: String,
    pub preview_url: Option<String>,
    pub album_image_url: Option<String>,
    
}

impl TrackInfo {
    pub async fn from_personal_track_info(
        personal: PersonalTrackInfo,
        spotify_client: &SpotifyClient,
    ) -> Result<Self, SpotifyError> {
        let spotify_track = spotify_client.get_track(personal.id.as_str()).await?;
        
        Ok(TrackInfo {
            artist: personal.artist,
            artist_id: personal.artist_id,
            title: personal.title,
            ms_duration: personal.ms_duration,
            times_played: personal.times_played,
            ms_played: personal.ms_played,
            time_distribution: personal.time_distribution,
            popularity: personal.popularity,
            spotify_url: spotify_track.external_urls.spotify,
            album_image_url: spotify_track.album.images.first().map(|img| img.url.clone()),
            preview_url: spotify_track.preview_url,
        })
    }
}