# Photolio

A self-hosted photography website with managed galleries, customizable site details and themes, a browser-based admin area, upload activity and persistent photo storage.

## Run with Docker Compose

Docker Compose builds the backend and frontend images directly. You do not need Make or a separate manual build command.

1. Copy the example environment file: `cp .env.example .env`
2. Edit `.env` and replace `PH_ADMIN_KEY` with a long, unique secret.
3. Start the services: `docker compose up -d --build`
4. Open `http://YOUR-SERVER-IP:8080` and visit `/admin` to sign in.

The webapp container serves the site and proxies `/api` requests to the backend. Only the webapp port is published. Originals, thumbnails and site settings are stored in the persistent Docker volume `photolio_data`.

To update after pulling changes, run `docker compose up -d --build`. Stop services without deleting photo data using `docker compose down`.

## Admin features

- Website title, tagline, about text, contact email and phone.
- Four themes: Classic, Dark, Warm and Editorial.
- Website, Instagram, YouTube, Facebook and TikTok links.
- Separate public galleries with titles, descriptions and individual `/gallery/SLUG` pages.
- Multi-photo uploads directly into a selected gallery.
- Recent activity feed and storage usage dashboard.

## 100 GB photo storage quota

The backend enforces a total photo quota of **100,000,000,000 bytes (100 GB decimal)** across the site's `content` directory. It checks available space before and while each upload is streamed to disk, and serializes upload requests so simultaneous uploads cannot use the same remaining quota.

Nginx accepts an individual request body up to 100 GB and streams it to the backend. The backend quota—not this per-request Nginx limit—enforces the total shared photo storage. Generated thumbnails and container logs are not counted in this quota.

## Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `PH_ADMINKEY` | required | Admin key checked by the backend |
| `PH_STORAGE_LOCATION` | `/var/data` | Persistent storage root |
| `PH_STORAGE_MAX_BYTES` | `100000000000` | Total photo quota in bytes |
| `PH_SERVER_ADDRESS` | `0.0.0.0` | Backend bind address |
| `PH_SERVER_PORT` | `80` | Backend port |
| `PHOTOLIO_PORT` | `8080` | Host port published by Compose |

For a public server, put Photolio behind HTTPS, for example Cloudflare Tunnel, and keep the admin key secret. The backend port is not published directly.

## Backups and multi-customer hosting

Back up the `photolio_data` Docker volume regularly. Do not remove it when updating containers.

The admin authentication in this edition uses one deployment-wide admin key. For a hosted business with independent customers, deploy one isolated Photolio instance and storage volume per customer. Customer accounts, tenant isolation and subscription management are not part of this single-site edition.
