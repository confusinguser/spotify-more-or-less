# Calendar Curator - Docker Setup

## Overview
Calendar Curator is a full-stack application with a Rust backend (Axum) and Next.js frontend, now fully containerized with Docker.

## Quick Start with Docker

### Prerequisites
- Docker and Docker Compose installed on your system

### Running the Application

1. **Clone and navigate to the project directory**
2. **Start the application**:
   ```bash
   docker-compose up --build
   ```

3. **Access the application**:
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8000

### Docker Services

- **Backend** (Rust/Axum): Runs on port 8000
- **Frontend** (Next.js): Runs on port 3000
- **Persistent storage**: Calendar data is stored in a Docker volume

### Development

To run in development mode:
```bash
# Start only the backend in Docker
docker-compose up backend

# Run frontend locally for hot reloading
cd web
npm install
npm run dev
```

### Docker Commands

```bash
# Build and start services
docker-compose up --build

# Start in background
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f

# Rebuild specific service
docker-compose build backend
docker-compose build frontend
```

### Data Persistence

Calendar data is persisted in a Docker volume named `calendar_data`. To backup or restore data:

```bash
# Backup
docker run --rm -v calendar_data:/data -v $(pwd):/backup alpine tar czf /backup/calendar-backup.tar.gz -C /data .

# Restore
docker run --rm -v calendar_data:/data -v $(pwd):/backup alpine tar xzf /backup/calendar-backup.tar.gz -C /data
```

### Environment Variables

- `DATABASE_PATH`: Path to the calendar database file (default: `/app/data/calendars.json`)
- `NODE_ENV`: Environment for Next.js (automatically set to `production` in Docker)

### Troubleshooting

1. **Port conflicts**: Ensure ports 3000 and 8000 are available
2. **Build issues**: Try `docker-compose build --no-cache`
3. **Data issues**: Check volume mounting with `docker volume inspect calendar_data`
