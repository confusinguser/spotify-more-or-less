# Spotify More Less

A full-stack game where users guess which song has more streams on Spotify.

## Quick Start

### Using Docker (Recommended)

1. **Set up your Spotify credentials**:
   ```bash
   export SPOTIFY_CLIENT_ID="your_client_id"
   export SPOTIFY_CLIENT_SECRET="your_client_secret"
   ```

2. **Place your data in the `./data` directory**:
   - Put your `tracks.json` OR
   - Put your Spotify extended history files (e.g., `StreamingHistory_music_0.json`)

3. **Start the application**:
   ```bash
   docker-compose up --build
   ```

4. **Access the game**:
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8000

### Running Locally

#### Backend (Rust)
```bash
cd app
export SPOTIFY_CLIENT_ID="your_client_id"
export SPOTIFY_CLIENT_SECRET="your_client_secret"
export DATA_DIR="./data"
cargo run --release
```

#### Frontend (Next.js)
```bash
cd web
pnpm install
pnpm dev
```

## Data Configuration

The application **automatically detects** the data format! Just point it to a directory containing your data files.

### Environment Variables

- **`DATA_DIR`** - Directory containing data files (recommended, auto-detects format)
- **`DATA_PATH`** - Specific file path or directory
- **`SPOTIFY_CLIENT_ID`** - Your Spotify API client ID
- **`SPOTIFY_CLIENT_SECRET`** - Your Spotify API client secret

### Supported Data Formats

#### 1. Extended Streaming History (Recommended)
Place your Spotify extended history JSON files in the data directory:
```
./data/
  StreamingHistory_music_0.json
  StreamingHistory_music_1.json
  StreamingHistory_music_2.json
```

The application will automatically detect and merge all files.

#### 2. Tracks JSON
A single aggregated JSON file:
```
./data/
  tracks.json
```

### Getting Your Spotify Data

1. Request your data: https://www.spotify.com/account/privacy/
2. Wait for the email (up to 30 days)
3. Download and extract the archive
4. Copy JSON files to `./data/` directory
5. Run the application!

## How It Works

### Auto-Detection
The application scans the configured directory and automatically detects:
- **Extended History Files**: JSON arrays with `ts`, `ms_played`, `spotify_track_uri` fields
- **Tracks JSON Files**: JSON objects with `spotify:track:*` keys

### Data Processing
When using extended history:
- Filters music tracks (excludes podcasts/audiobooks)
- Aggregates statistics per track
- Calculates play counts and time distributions
- Fetches additional metadata from Spotify API

## Game Features

- **3 Lives System**: Get 3 chances before game over
- **Dramatic Animations**: Cards fly away with various animations
- **Real-time Stats**: Track your score and high score
- **Progressive Difficulty**: Uses your actual listening history

## Docker Configuration

### docker-compose.yml
```yaml
services:
  app:
    build: .
    ports:
      - "3000:3000"  # Frontend
      - "8000:8000"  # Backend API
    volumes:
      - ./data:/app/data  # Mount your data directory
    environment:
      - DATA_DIR=/app/data
      - SPOTIFY_CLIENT_ID=${SPOTIFY_CLIENT_ID}
      - SPOTIFY_CLIENT_SECRET=${SPOTIFY_CLIENT_SECRET}
```

### Environment File (.env)
Create a `.env` file:
```env
SPOTIFY_CLIENT_ID=your_client_id_here
SPOTIFY_CLIENT_SECRET=your_client_secret_here
```

## Development

### Backend Tests
```bash
cd app
cargo test
```

### Frontend Development
```bash
cd web
pnpm dev
```

### Building for Production
```bash
# Backend
cd app
cargo build --release

# Frontend
cd web
pnpm build
```

## API Endpoints

- `GET /tracks/random` - Get a random track
- `GET /tracks/random/two` - Get two random tracks
- `GET /tracks/{track_id}/album-image` - Get album image URL
- `GET /openapi.json` - OpenAPI schema

## Troubleshooting

### No data found
- Ensure your data directory contains valid JSON files
- Check file permissions
- Look for error messages in console output

### Backend fails to start
- Verify Spotify credentials are set
- Check if port 8000 is available
- Ensure data files are in the correct format

### Frontend can't connect to backend
- Ensure backend is running on port 8000
- Check CORS configuration
- Verify network connectivity

## Architecture

```
┌─────────────────┐
│   Next.js       │  Frontend (Port 3000)
│   Frontend      │
└────────┬────────┘
         │
         │ API Calls
         │
┌────────▼────────┐
│   Axum          │  Backend (Port 8000)
│   Rust Backend  │
└────────┬────────┘
         │
         │ OAuth2
         │
┌────────▼────────┐
│   Spotify API   │  External Service
└─────────────────┘
```

## License

See LICENSE file for details.

## Contributing

Contributions welcome! Please open an issue or PR.

