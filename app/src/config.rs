use crate::extended_history;
use crate::storage::Storage;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Configuration for data source format
#[derive(Debug, Clone)]
pub enum DataSourceConfig {
    /// Use the tracks.json format (single file)
    TracksJson { path: PathBuf },
    /// Use extended streaming history format (single file)
    ExtendedHistorySingle { path: PathBuf },
    /// Use extended streaming history format (multiple files)
    ExtendedHistoryMultiple { paths: Vec<PathBuf> },
    /// Auto-detect format from a directory
    AutoDetectDirectory { path: PathBuf },
}

/// Detected file type
#[derive(Debug, Clone, PartialEq)]
enum FileType {
    TracksJson,
    ExtendedHistory,
    Unknown,
}

/// Detect the file type by examining its content
fn detect_file_type<P: AsRef<Path>>(path: P) -> io::Result<FileType> {
    let content = fs::read_to_string(path.as_ref())?;

    // Try to parse a small portion to detect structure
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
        match value {
            // TracksJson: Object with track URIs as keys
            serde_json::Value::Object(map) => {
                if map.keys().any(|k| k.starts_with("spotify:track:")) {
                    return Ok(FileType::TracksJson);
                }
            }
            // ExtendedHistory: Array of entries
            serde_json::Value::Array(arr) => {
                if !arr.is_empty() {
                    if let Some(obj) = arr[0].as_object() {
                        // Check for extended history signature fields
                        if obj.contains_key("ts")
                            && obj.contains_key("ms_played")
                            && (obj.contains_key("spotify_track_uri") || obj.contains_key("master_metadata_track_name"))
                        {
                            return Ok(FileType::ExtendedHistory);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(FileType::Unknown)
}

/// Find all JSON files in a directory and detect their types
fn scan_directory<P: AsRef<Path>>(dir_path: P) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut tracks_json_files = Vec::new();
    let mut extended_history_files = Vec::new();

    if !dir_path.as_ref().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Not a directory: {:?}", dir_path.as_ref()),
        ));
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match detect_file_type(&path)? {
                FileType::TracksJson => tracks_json_files.push(path),
                FileType::ExtendedHistory => extended_history_files.push(path),
                FileType::Unknown => {
                    eprintln!("Warning: Unknown JSON format in file: {:?}", path);
                }
            }
        }
    }

    Ok((tracks_json_files, extended_history_files))
}

impl DataSourceConfig {
    /// Load Storage based on the configuration
    pub fn load_storage(&self, min_streams: u32) -> io::Result<Storage> {
        match self {
            DataSourceConfig::TracksJson { path } => {
                Storage::from_file(path, min_streams)
            }
            DataSourceConfig::ExtendedHistorySingle { path } => {
                extended_history::load_storage_from_extended_history(path, min_streams)
            }
            DataSourceConfig::ExtendedHistoryMultiple { paths } => {
                extended_history::load_storage_from_extended_history_files(paths, min_streams)
            }
            DataSourceConfig::AutoDetectDirectory { path } => {
                let (tracks_json_files, extended_history_files) = scan_directory(path)?;

                // Priority: Use extended history if available (more detailed)
                if !extended_history_files.is_empty() {
                    println!("Auto-detected {} extended history file(s)", extended_history_files.len());
                    extended_history::load_storage_from_extended_history_files(&extended_history_files, min_streams)
                } else if !tracks_json_files.is_empty() {
                    println!("Auto-detected {} tracks.json file(s)", tracks_json_files.len());
                    if tracks_json_files.len() == 1 {
                        Storage::from_file(&tracks_json_files[0], min_streams)
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Multiple tracks.json files found. Expected only one.",
                        ));
                    }
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "No recognized data files found in directory",
                    ))
                }
            }
        }
    }

    /// Create configuration from a directory path (auto-detect files)
    pub fn from_directory<P: Into<PathBuf>>(path: P) -> Self {
        DataSourceConfig::AutoDetectDirectory {
            path: path.into(),
        }
    }

    /// Create configuration from environment variable
    ///
    /// Supports:
    /// - DATA_DIR: Directory containing data files (auto-detect)
    /// - DATA_PATH: Specific file path
    pub fn from_env() -> Self {
        // Check for directory first (simplest option)
        if let Ok(dir_path) = std::env::var("DATA_DIR") {
            return DataSourceConfig::AutoDetectDirectory {
                path: PathBuf::from(dir_path),
            };
        }

        // Fall back to specific file path
        let path = std::env::var("DATA_PATH")
            .or_else(|_| std::env::var("data_path"))
            .unwrap_or_else(|_| "./data".to_string());

        let path_buf = PathBuf::from(&path);

        // If it's a directory, auto-detect
        if path_buf.is_dir() {
            DataSourceConfig::AutoDetectDirectory { path: path_buf }
        } else {
            // Assume it's a specific file, default to TracksJson
            DataSourceConfig::TracksJson { path: path_buf }
        }
    }
}

/// Builder for DataSourceConfig to make configuration easier
pub struct DataSourceBuilder {
    config: Option<DataSourceConfig>,
}

impl DataSourceBuilder {
    pub fn new() -> Self {
        DataSourceBuilder { config: None }
    }

    /// Auto-detect files in a directory
    pub fn with_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config = Some(DataSourceConfig::AutoDetectDirectory {
            path: path.into(),
        });
        self
    }

    /// Use tracks.json format
    pub fn with_tracks_json<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config = Some(DataSourceConfig::TracksJson {
            path: path.into(),
        });
        self
    }

    /// Use extended history format (single file)
    pub fn with_extended_history<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config = Some(DataSourceConfig::ExtendedHistorySingle {
            path: path.into(),
        });
        self
    }

    /// Use extended history format (multiple files)
    pub fn with_extended_history_files<P: Into<PathBuf>>(
        mut self,
        paths: Vec<P>,
    ) -> Self {
        self.config = Some(DataSourceConfig::ExtendedHistoryMultiple {
            paths: paths.into_iter().map(|p| p.into()).collect(),
        });
        self
    }

    /// Use environment variables to determine configuration
    pub fn use_env(mut self) -> Self {
        self.config = Some(DataSourceConfig::from_env());
        self
    }

    /// Build the configuration
    pub fn build(self) -> DataSourceConfig {
        self.config.unwrap_or_else(DataSourceConfig::from_env)
    }
}

impl Default for DataSourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_directory() {
        let config = DataSourceBuilder::new()
            .with_directory("./data")
            .build();

        match config {
            DataSourceConfig::AutoDetectDirectory { path } => {
                assert_eq!(path, PathBuf::from("./data"));
            }
            _ => panic!("Expected AutoDetectDirectory variant"),
        }
    }

    #[test]
    fn test_builder_tracks_json() {
        let config = DataSourceBuilder::new()
            .with_tracks_json("./data/tracks.json")
            .build();

        match config {
            DataSourceConfig::TracksJson { path } => {
                assert_eq!(path, PathBuf::from("./data/tracks.json"));
            }
            _ => panic!("Expected TracksJson variant"),
        }
    }

    #[test]
    fn test_builder_extended_history_single() {
        let config = DataSourceBuilder::new()
            .with_extended_history("./data/history.json")
            .build();

        match config {
            DataSourceConfig::ExtendedHistorySingle { path } => {
                assert_eq!(path, PathBuf::from("./data/history.json"));
            }
            _ => panic!("Expected ExtendedHistorySingle variant"),
        }
    }

    #[test]
    fn test_builder_extended_history_multiple() {
        let paths = vec!["./data/history1.json", "./data/history2.json"];
        let config = DataSourceBuilder::new()
            .with_extended_history_files(paths)
            .build();

        match config {
            DataSourceConfig::ExtendedHistoryMultiple { paths } => {
                assert_eq!(paths.len(), 2);
                assert_eq!(paths[0], PathBuf::from("./data/history1.json"));
                assert_eq!(paths[1], PathBuf::from("./data/history2.json"));
            }
            _ => panic!("Expected ExtendedHistoryMultiple variant"),
        }
    }

    #[test]
    fn test_from_directory() {
        let config = DataSourceConfig::from_directory("./data");

        match config {
            DataSourceConfig::AutoDetectDirectory { path } => {
                assert_eq!(path, PathBuf::from("./data"));
            }
            _ => panic!("Expected AutoDetectDirectory variant"),
        }
    }
}

