# Data Source Configuration Guide

This application supports multiple data source formats for loading your Spotify listening history. You can easily switch between formats using environment variables or programmatic configuration.

## Supported Formats

### 1. TracksJson Format (Default)

A single JSON file containing aggregated track statistics.

**Structure:**
```json
{
  "spotify:track:6BAnxKyld909yo6Pk1DO3r": {
    "Artist": "OMFG",
    "ArtistID": "artist_id",
    "Title": "Hello",
    "msDuration": 180000,
    "TimesPlayed": 42,
    "msPlayed": 7560000,
    "timeDistribution": [0, 0, 1, 2, 5, ...],
    "Popularity": 75
  }
}
```

**Usage:**
```bash
# Environment variable
export DATA_PATH="./data/tracks.json"

# Or
export data_path="./data/tracks.json"
```

### 2. Extended Streaming History Format

Spotify's raw extended streaming history export format. Supports both single file and multiple files.

**Structure:**
```json
[
  {
    "ts": "2017-05-25T14:42:15Z",
    "platform": "Android OS 6.0 API 23 (HUAWEI, PLK-L01)",
    "ms_played": 48114,
    "conn_country": "SE",
    "master_metadata_track_name": "Hello",
    "master_metadata_album_artist_name": "OMFG",
    "spotify_track_uri": "spotify:track:6BAnxKyld909yo6Pk1DO3r",
    "reason_start": "playbtn",
    "reason_end": "endplay",
    "shuffle": true,
    "skipped": false,
    ...
  }
]
```

#### Single File
```bash
export EXTENDED_HISTORY_PATH="./data/streaming_history.json"
```

#### Multiple Files
```bash
export EXTENDED_HISTORY_PATHS="./data/history1.json,./data/history2.json,./data/history3.json"
```

## Environment Variable Priority

The application checks environment variables in the following order:

1. **EXTENDED_HISTORY_PATHS** - Comma-separated list of extended history files
2. **EXTENDED_HISTORY_PATH** - Single extended history file
3. **DATA_PATH** or **data_path** - TracksJson format file
4. **Default** - `./data/tracks.json`

## Programmatic Configuration

### Using DataSourceConfig Directly

```rust
use crate::config::DataSourceConfig;

// From environment variables (recommended)
let config = DataSourceConfig::from_env();
let storage = config.load_storage()?;

// Explicit TracksJson
let config = DataSourceConfig::TracksJson {
    path: PathBuf::from("./data/tracks.json")
};
let storage = config.load_storage()?;

// Single extended history file
let config = DataSourceConfig::ExtendedHistorySingle {
    path: PathBuf::from("./data/history.json")
};
let storage = config.load_storage()?;

// Multiple extended history files
let config = DataSourceConfig::ExtendedHistoryMultiple {
    paths: vec![
        PathBuf::from("./data/history1.json"),
        PathBuf::from("./data/history2.json"),
    ]
};
let storage = config.load_storage()?;
```

### Using DataSourceBuilder (Fluent API)

```rust
use crate::config::DataSourceBuilder;

// From environment
let storage = DataSourceBuilder::new()
    .use_env()
    .build()
    .load_storage()?;

// TracksJson format
let storage = DataSourceBuilder::new()
    .with_tracks_json("./data/tracks.json")
    .build()
    .load_storage()?;

// Extended history (single file)
let storage = DataSourceBuilder::new()
    .with_extended_history("./data/history.json")
    .build()
    .load_storage()?;

// Extended history (multiple files)
let storage = DataSourceBuilder::new()
    .with_extended_history_files(vec![
        "./data/history1.json",
        "./data/history2.json",
        "./data/history3.json",
    ])
    .build()
    .load_storage()?;
```

## Extended History Conversion Details

When loading from extended streaming history, the application:

1. **Filters** - Only processes music tracks (excludes podcasts and audiobooks)
2. **Aggregates** - Combines multiple plays of the same track
3. **Calculates**:
   - Total play count per track
   - Total milliseconds played
   - Time distribution (plays per hour of day, 0-23)
4. **Limitations**:
   - Track duration set to 0 (not in extended history)
   - Popularity set to 0 (not in extended history)
   - Artist ID left empty (not in extended history)

These fields can be enriched later by calling the Spotify API.

## Running the Application

### With TracksJson
```bash
DATA_PATH="./data/tracks.json" cargo run --release
```

### With Extended History (Single File)
```bash
EXTENDED_HISTORY_PATH="./data/StreamingHistory_music_0.json" cargo run --release
```

### With Extended History (Multiple Files)
```bash
EXTENDED_HISTORY_PATHS="./data/StreamingHistory_music_0.json,./data/StreamingHistory_music_1.json,./data/StreamingHistory_music_2.json" cargo run --release
```

### Docker
```bash
# Set in docker-compose.yml or pass as environment variables
docker-compose up --build
```

## Converting Between Formats

You can also use the conversion functions directly:

```rust
use crate::extended_history;

// Load and convert
let history = extended_history::load_extended_history_from_file("./data/history.json")?;
let storage = extended_history::extended_history_to_storage(history);

// Multiple files
let history = extended_history::load_extended_history_from_files(&[
    "./data/history1.json",
    "./data/history2.json",
])?;
let storage = extended_history::extended_history_to_storage(history);

// Or use the convenience functions
let storage = extended_history::load_storage_from_extended_history("./data/history.json")?;
let storage = extended_history::load_storage_from_extended_history_files(&[
    "./data/history1.json",
    "./data/history2.json",
])?;
```

## Testing

The application includes unit tests for all conversion logic:

```bash
# Run all tests
cargo test

# Run specific tests
cargo test extended_history
cargo test config
```

## Example: Getting Your Spotify Data

1. Request your data from Spotify: https://www.spotify.com/account/privacy/
2. Wait for the email (can take up to 30 days)
3. Download and extract the archive
4. Look for files like `StreamingHistory_music_0.json`, `StreamingHistory_music_1.json`, etc.
5. Set the environment variable and run:

```bash
EXTENDED_HISTORY_PATHS="path/to/StreamingHistory_music_0.json,path/to/StreamingHistory_music_1.json" cargo run --release
```

