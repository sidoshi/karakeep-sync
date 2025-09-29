# Karakeep Sync Docker Deployment

This repository provides a lean Docker setup for the `karakeep-sync` Rust binary.

## Quick Start

1. **Clone or download the `docker-compose.yml` file**
2. **Create your environment file:**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```
3. **Run the service:**
   ```bash
   docker-compose up -d
   ```

## Configuration

Configure the application by editing the `.env` file or by setting environment variables directly in the `docker-compose.yml` file.

### Environment Variables

- `RUST_LOG`: Set logging level (default: `info`)
- Add your application-specific environment variables as needed

## Docker Image

The Docker image is built using a multi-stage build process:
- **Builder stage**: Uses `rust:1.79-slim` to compile the binary
- **Runtime stage**: Uses `gcr.io/distroless/cc-debian12` for minimal size and security

Image size: ~20-30MB (compared to ~1GB+ with standard Rust images)

## Commands

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f karakeep-sync

# Stop the service
docker-compose down

# Update to latest image
docker-compose pull
docker-compose up -d
```

## Building Locally

If you want to build the image locally instead of using the pre-built one:

```bash
# Build the image
docker build -t karakeep-sync .

# Update docker-compose.yml to use local image
# Change: image: ghcr.io/sidoshi/karakeep-sync:latest
# To: image: karakeep-sync:latest
```

## Security

- Runs as non-root user
- Uses distroless base image (no shell, minimal attack surface)
- Resource limits configured in docker-compose.yml
