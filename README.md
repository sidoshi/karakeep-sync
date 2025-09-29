# karakeep-sync

A tool to sync links from various services to [Karakeep](https://github.com/hoarder-app/hoarder) to keep all your interesting links in one place.

## Overview

When looking up something interesting you found in the past, you probably check multiple places - Karakeep, HN upvotes, Reddit bookmarks, etc. This tool syncs all those links to Karakeep automatically, organizing them under lists for easy future access.

## Supported Services

- ✅ Hacker News upvotes
- 🚧 More services coming soon

## Environment Variables

Configure these environment variables in your `docker-compose.yml`:

| Variable | Required | Description |
|----------|----------|-------------|
| `KS_KARAKEEP_AUTH` | ✅ | Your Karakeep API token |
| `KS_KARAKEEP_URL` | ✅ | Your Karakeep instance URL (e.g., `https://karakeep.example.com`) |
| `KS_HN_AUTH` | ✅ | Your Hacker News authentication cookie value |
| `KS_HN_SCHEDULE` | ❌ | Sync schedule in cron format (default: `@daily`) |
| `RUST_LOG` | ❌ | Log level (default: `info`) |

## Deployment

Create a `docker-compose.yml` file with the following content:

```yaml
services:
  karakeep-sync:
    image: ghcr.io/sidoshi/karakeep-sync:latest
    container_name: karakeep-sync
    restart: unless-stopped
    environment:
      - KS_KARAKEEP_AUTH=<your_karakeep_auth_cookie> # required
      - KS_KARAKEEP_URL=<your_karakeep_instance_url> # required
      - KS_HN_AUTH=<your_hn_auth_cookie> # required
      - KS_HN_SCHEDULE=@daily # Cron format, e.g., "@hourly", "@daily", "0 0 * * *" default is "@daily"
```

Then run:

```bash
docker-compose up -d
```

You can also add this service definition alongside your existing Hoarder/Karakeep services.
