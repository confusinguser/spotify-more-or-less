use crate::storage::{PersonalTrackInfo, Storage, TracksJson};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// A single streaming history entry from Spotify's extended streaming history export
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtendedHistoryEntry {
    /// Timestamp when the track was played
    pub ts: String,
    /// Platform used for playback
    pub platform: String,
    /// Milliseconds played
    pub ms_played: u64,
    /// Country code where the track was played
    pub conn_country: String,
    /// IP address (optional for privacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_addr: Option<String>,
    /// Track name
    pub master_metadata_track_name: Option<String>,
    /// Artist name
    pub master_metadata_album_artist_name: Option<String>,
    /// Album name
    pub master_metadata_album_album_name: Option<String>,
    /// Spotify track URI (e.g., "spotify:track:6BAnxKyld909yo6Pk1DO3r")
    pub spotify_track_uri: Option<String>,
    /// Episode name (for podcasts)
    pub episode_name: Option<String>,
    /// Episode show name (for podcasts)
    pub episode_show_name: Option<String>,
    /// Spotify episode URI
    pub spotify_episode_uri: Option<String>,
    /// Audiobook title
    pub audiobook_title: Option<String>,
    /// Audiobook URI
    pub audiobook_uri: Option<String>,
    /// Audiobook chapter URI
    pub audiobook_chapter_uri: Option<String>,
    /// Audiobook chapter title
    pub audiobook_chapter_title: Option<String>,
    /// Reason playback started
    pub reason_start: String,
    /// Reason playback ended
    pub reason_end: String,
    /// Whether shuffle was enabled
    pub shuffle: bool,
    /// Whether the track was skipped
    #[serde(default)]
    pub skipped: Option<bool>,
    /// Whether played offline
    pub offline: bool,
    /// Offline timestamp
    pub offline_timestamp: Option<u64>,
    /// Whether incognito mode was enabled
    pub incognito_mode: bool,
}

/// The root type for extended streaming history (array of entries)
pub type ExtendedHistory = Vec<ExtendedHistoryEntry>;

impl ExtendedHistoryEntry {
    /// Extract the Spotify track ID from the URI
    /// URI format: "spotify:track:6BAnxKyld909yo6Pk1DO3r"
    pub fn extract_track_id(&self) -> Option<String> {
        self.spotify_track_uri
            .as_ref()
            .and_then(|uri| uri.strip_prefix("spotify:track:"))
            .map(|id| id.to_string())
    }

    /// Get the hour of day when this track was played (0-23)
    pub fn get_hour_of_day(&self) -> Option<usize> {
        // Parse timestamp format: "2017-05-25T14:42:15Z"
        self.ts
            .split('T')
            .nth(1)
            .and_then(|time_part| time_part.split(':').next())
            .and_then(|hour_str| hour_str.parse::<usize>().ok())
            .filter(|&hour| hour < 24)
    }

    /// Check if this entry represents a music track (not podcast/audiobook)
    pub fn is_music_track(&self) -> bool {
        self.spotify_track_uri.is_some()
            && self.episode_name.is_none()
            && self.audiobook_title.is_none()
    }
}

/// Statistics aggregated for a single track from extended history
#[derive(Debug, Clone)]
struct TrackStats {
    track_id: String,
    artist: String,
    title: String,
    times_played: u32,
    ms_played: u64,
    time_distribution: Vec<u32>,
    // We don't have duration or popularity in extended history
    // These will need to be filled in later or set to defaults
}

impl TrackStats {
    /// Create a new TrackStats from an extended history entry
    fn new(entry: &ExtendedHistoryEntry, track_id: String) -> Self {
        let mut time_distribution = vec![0; 24];
        if let Some(hour) = entry.get_hour_of_day() {
            time_distribution[hour] = 1;
        }

        TrackStats {
            track_id,
            artist: entry
                .master_metadata_album_artist_name
                .clone()
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            title: entry
                .master_metadata_track_name
                .clone()
                .unwrap_or_else(|| "Unknown Track".to_string()),
            times_played: 1,
            ms_played: entry.ms_played,
            time_distribution,
        }
    }

    /// Update stats with a new play entry
    fn add_play(&mut self, entry: &ExtendedHistoryEntry) {
        self.times_played += 1;
        self.ms_played += entry.ms_played;

        if let Some(hour) = entry.get_hour_of_day() {
            self.time_distribution[hour] += 1;
        }
    }

    /// Convert to PersonalTrackInfo
    fn to_personal_track_info(self) -> PersonalTrackInfo {
        PersonalTrackInfo {
            id: self.track_id.clone(),
            artist: self.artist,
            artist_id: String::new(), // Extended history doesn't include artist ID
            title: self.title,
            ms_duration: 0, // Extended history doesn't include track duration
            times_played: self.times_played,
            ms_played: self.ms_played,
            time_distribution: self.time_distribution,
            popularity: 0, // Extended history doesn't include popularity
        }
    }
}

/// Aggregate extended history entries into track statistics
fn aggregate_track_stats(history: ExtendedHistory) -> HashMap<String, TrackStats> {
    let mut track_stats: HashMap<String, TrackStats> = HashMap::new();

    for entry in history {
        // Skip non-music entries (podcasts, audiobooks)
        if !entry.is_music_track() {
            continue;
        }

        // Extract track ID and metadata
        let Some(track_id) = entry.extract_track_id() else {
            continue;
        };

        // Update or create track statistics
        track_stats
            .entry(track_id.clone())
            .and_modify(|stats| stats.add_play(&entry))
            .or_insert_with(|| TrackStats::new(&entry, track_id));
    }

    track_stats
}

/// Convert aggregated track stats to TracksJson
fn track_stats_to_tracks_json(track_stats: HashMap<String, TrackStats>) -> TracksJson {
    let mut map = IndexMap::new();

    for (track_id, stats) in track_stats {
        let personal_track_info = stats.to_personal_track_info();
        map.insert(track_id, personal_track_info);
    }

    TracksJson::from_map(map)
}

/// Convert extended streaming history to Storage
///
/// This function processes the extended history and aggregates play statistics
/// for each track. Note that extended history doesn't include track duration
/// or popularity, so these fields will be set to 0.
pub fn extended_history_to_storage(history: ExtendedHistory, min_streams: u32) -> Storage {
    let track_stats = aggregate_track_stats(history);
    let tracks_json = track_stats_to_tracks_json(track_stats);
    let filtered_tracks_json = {
        let mut filtered_map = IndexMap::new();
        for (track_id, track_info) in tracks_json.map.into_iter() {
            if track_info.times_played >= min_streams {
                filtered_map.insert(track_id, track_info);
            }
        }
        TracksJson::from_map(filtered_map)
    };
    Storage::from_tracks_json(filtered_tracks_json)
}

/// Load extended history from a single JSON file
pub fn load_extended_history_from_file<P: AsRef<Path>>(
    path: P,
) -> io::Result<ExtendedHistory> {
    let file = std::fs::File::open(path)?;
    let history: ExtendedHistory = serde_json::from_reader(file)?;
    Ok(history)
}

/// Load extended history from multiple JSON files and merge them
///
/// Each file should contain a complete JSON array of extended history entries.
/// All entries from all files will be combined into a single vector.
pub fn load_extended_history_from_files<P: AsRef<Path>>(
    paths: &[P],
) -> io::Result<ExtendedHistory> {
    let mut combined_history = Vec::new();

    for path in paths {
        let mut file_history = load_extended_history_from_file(path)?;
        combined_history.append(&mut file_history);
    }

    Ok(combined_history)
}

/// Load extended history from a single JSON file and convert to Storage
pub fn load_storage_from_extended_history<P: AsRef<Path>>(
    path: P,
    min_streams:u32
) -> io::Result<Storage> {
    let history = load_extended_history_from_file(path)?;
    Ok(extended_history_to_storage(history, min_streams))
}

/// Load extended history from multiple JSON files and convert to Storage
///
/// This function merges all extended history files and aggregates the statistics
/// across all of them into a single Storage object.
pub fn load_storage_from_extended_history_files<P: AsRef<Path>>(
    paths: &[P],
    min_streams: u32,
) -> io::Result<Storage> {
    let history = load_extended_history_from_files(paths)?;
    Ok(extended_history_to_storage(history, min_streams))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_track_id() {
        let entry = ExtendedHistoryEntry {
            ts: "2017-05-25T14:42:15Z".to_string(),
            platform: "Android".to_string(),
            ms_played: 48114,
            conn_country: "SE".to_string(),
            ip_addr: Some("83.254.152.142".to_string()),
            master_metadata_track_name: Some("Hello".to_string()),
            master_metadata_album_artist_name: Some("OMFG".to_string()),
            master_metadata_album_album_name: Some("Hello".to_string()),
            spotify_track_uri: Some("spotify:track:6BAnxKyld909yo6Pk1DO3r".to_string()),
            episode_name: None,
            episode_show_name: None,
            spotify_episode_uri: None,
            audiobook_title: None,
            audiobook_uri: None,
            audiobook_chapter_uri: None,
            audiobook_chapter_title: None,
            reason_start: "playbtn".to_string(),
            reason_end: "endplay".to_string(),
            shuffle: true,
            skipped: Some(false),
            offline: false,
            offline_timestamp: None,
            incognito_mode: false,
        };

        assert_eq!(
            entry.extract_track_id(),
            Some("6BAnxKyld909yo6Pk1DO3r".to_string())
        );
    }

    #[test]
    fn test_get_hour_of_day() {
        let entry = ExtendedHistoryEntry {
            ts: "2017-05-25T14:42:15Z".to_string(),
            platform: "Android".to_string(),
            ms_played: 48114,
            conn_country: "SE".to_string(),
            ip_addr: None,
            master_metadata_track_name: Some("Hello".to_string()),
            master_metadata_album_artist_name: Some("OMFG".to_string()),
            master_metadata_album_album_name: Some("Hello".to_string()),
            spotify_track_uri: Some("spotify:track:6BAnxKyld909yo6Pk1DO3r".to_string()),
            episode_name: None,
            episode_show_name: None,
            spotify_episode_uri: None,
            audiobook_title: None,
            audiobook_uri: None,
            audiobook_chapter_uri: None,
            audiobook_chapter_title: None,
            reason_start: "playbtn".to_string(),
            reason_end: "endplay".to_string(),
            shuffle: true,
            skipped: Some(false),
            offline: false,
            offline_timestamp: None,
            incognito_mode: false,
        };

        assert_eq!(entry.get_hour_of_day(), Some(14));
    }

    #[test]
    fn test_is_music_track() {
        let music_entry = ExtendedHistoryEntry {
            ts: "2017-05-25T14:42:15Z".to_string(),
            platform: "Android".to_string(),
            ms_played: 48114,
            conn_country: "SE".to_string(),
            ip_addr: None,
            master_metadata_track_name: Some("Hello".to_string()),
            master_metadata_album_artist_name: Some("OMFG".to_string()),
            master_metadata_album_album_name: Some("Hello".to_string()),
            spotify_track_uri: Some("spotify:track:6BAnxKyld909yo6Pk1DO3r".to_string()),
            episode_name: None,
            episode_show_name: None,
            spotify_episode_uri: None,
            audiobook_title: None,
            audiobook_uri: None,
            audiobook_chapter_uri: None,
            audiobook_chapter_title: None,
            reason_start: "playbtn".to_string(),
            reason_end: "endplay".to_string(),
            shuffle: true,
            skipped: Some(false),
            offline: false,
            offline_timestamp: None,
            incognito_mode: false,
        };

        assert!(music_entry.is_music_track());
    }
}

