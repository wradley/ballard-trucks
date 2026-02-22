# ballard-trucks
Daily forecast of the local Ballard food trucks.

Public backend API routes are read-only. Python scrapers write through an internal authenticated backend ingest endpoint.

## Local Docker test (backend)
```sh
docker build -f backend/Dockerfile -t ballard-backend-dev .
docker run --rm -p 8080:8080 -e SQLITE_PATH=/data/ballard.sqlite -v ballard-data:/data ballard-backend-dev
```

In another terminal:
```sh
curl http://localhost:8080/api/health
```

## Local Stack (Docker Compose)
```sh
docker compose up --build
```

Ingest auth and scraper-to-backend wiring (defaults in `compose.yml`):
- `INGEST_API_KEY` is required by backend and scrapers for internal ingest auth.
- `BACKEND_INGEST_URL` is used by scrapers to reach backend ingest (default: `http://backend:8080`).

Example with an explicit ingest key:
```sh
INGEST_API_KEY="$(uuidgen)" docker compose up --build
```

## Local Backend Testing (Host Runtime)
Export the SQLite file from the Docker volume to a local snapshot:
```sh
./scripts/export-db-snapshot.sh
```

Run the backend locally against that snapshot:
```sh
cargo run --manifest-path backend/Cargo.toml -- --sqlite-path ./.local-db/ballard.sqlite
```

Smoke-test endpoints:
```sh
curl http://localhost:8080/api/health
curl http://localhost:8080/api/breweries
curl http://localhost:8080/api/vendors
curl "http://localhost:8080/api/schedules?start_hour_utc=2026-05-08T15&duration_hours=12"
```

## Local Scrapers (Docker)
Build the scraper image:
```sh
docker compose build scrapers
```

Run the scrapers:
```sh
docker compose run --rm scrapers
```

Scrapers do not mount or access SQLite directly. They send batches to backend `/internal/ingest/schedules`.

## Backend Tests
Run full backend tests:
```sh
cargo test --manifest-path backend/Cargo.toml
```

Run the ingest flow integration test only:
```sh
cargo test --manifest-path backend/Cargo.toml --test ingest_flow
```

If you want to reset local SQLite data:
```sh
docker compose down -v
```

See `spec/00-overview.md` for the current plan and phase breakdown.
See `openapi.yaml` for the current API contract (source of truth for request/response shape).

## High Level Design
![Architecture Diagram](/docs/trucks-arch.png)
