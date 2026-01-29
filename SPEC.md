# Ballard Trucks - Spec

## Goal
Build a web app that shows a forecast of food trucks at breweries in Ballard, Seattle.

## (Phase 1) Backend & Test Deployments
- [x] Hardcoded sample breweries: Stoup Brewing, Bale Breaker x Yonder, Urban Family.
  - Current state: seeded sample breweries are in place for Phase 1 development.
- [x] Sample schedule data (no scraping yet).
- [x] Backend API only.
- [x] Request correlation IDs in logs (`X-Request-Id` propagation/generation) for per-request tracing.
- [x] Unit tests for Phase 1 API/domain behavior (including schedule query parameter validation).

### Data Model (Initial)
Brewery:
- id (string)
- name
- address
- lat
- lng
- website

ScheduleEntry:
- id (string)
- brewery_id
- brewery_name
- truck_name
- date (YYYY-MM-DD)
- start_time (HH:MM)
- end_time (HH:MM)
- source (string)
- updated_at (RFC3339)

### API (Phase 1)
- [x] GET /api/health -> "ok"
  - Current state: implemented, returns `"ok"`.
- [x] GET /api/schedules?start_hour_utc=YYYY-MM-DDTHH&duration_hours=N -> ScheduleEntry[] (up to 100)
- [x] GET /api/breweries -> Brewery[]
- [x] GET /api/vendors -> Vendor[]

`GET /api/schedules` request contract (Phase 1 baseline):
- [x] `start_hour_utc` is a URL-encoded UTC hour bucket formatted as `YYYY-MM-DDTHH`.
- [x] `duration_hours` is an integer window size in hours.
- [x] API timestamps are UTC-only; client converts for local display.
  - Current state: schedule responses include UTC RFC3339 timestamps (`start_at`, `end_at`, `updated_at`).
- [x] Data updates regularly; schedule data more than 7 days ahead may be unavailable.
  - Current state: documentation guidance only (not server-enforced).
- [x] Response size is capped at 100 rows (pagination deferred to Phase 7).

### Deployment (Phase 1)
- Build and run backend locally with a minimal Rust binary (no web server yet).
- Confirm the binary runs in the local server environment.
- For multi-arch Docker builds (local testing on Apple Silicon and x86_64 servers):
  - `docker buildx build --platform linux/arm64 -t ballard-backend:arm64 backend`
  - `docker buildx build --platform linux/amd64 -t ballard-backend:amd64 backend`
  - `docker run --rm ballard-backend:arm64`
- Optional later phase: set up a local Docker registry on the home network for fast LAN pulls.
- Local Docker test commands are documented in `README.md`.

## Scope (Phase 2)
- Postgres-backed schedule cache is in place.
- Deployment test to local server environment.
- Add scheduled daily scraping + on-request refresh queue.
- Refine `GET /api/schedules` with optional ID-based filters:
  - `brewery_ids` (comma-separated IDs)
  - `vendor_ids` (comma-separated IDs)
- Support narrow lookups (for example, all locations for a single vendor over a date window).
- Unknown `brewery_ids` / `vendor_ids` are valid and return zero matching rows (no error).

### Data Model (Phase 2)
Vendor:
- id (string)
- name
- website (optional)

### Scraping Strategy (Phase 2+)
- Prefer APIs / calendar feeds / JSON-LD over HTML parsing.
- Cache daily snapshots and keep metadata (source URL, last-modified).
- Rate limit and respect robots.txt.

### Refresh Policy (Phase 2+)
- Store `last_scraped_at` and `refresh_requested_at` in the cache DB.
- Backend sets `refresh_requested_at = now()` only if:
  - data is stale beyond a `min_stale` threshold, and
  - `now - refresh_requested_at` exceeds a `min_request_interval` (dedupe).
- Scraper runs on a fixed schedule (e.g., twice/day or every 15 minutes) and:
  - acquires a Postgres advisory lock to ensure a single scraper instance,
  - checks if a refresh is requested or scheduled,
  - scrapes, updates `last_scraped_at`, and clears `refresh_requested_at`.

## Scope (Phase 3)
- Frontend setup (React) with date dropdown and list view only.

## Scope (Phase 4)
- Lookup by food truck name.

## Scope (Phase 5)
- Map view (Leaflet + OSM or alternative tile provider).

## Scope (Phase 6)
- UI beautification / styling.

## Scope (Phase 7)
- Stored user data (favorites, saved filters, and related user preferences).
- Metrics and observability dashboard for API health and usage trends.
  - Track schedule query volume and schedules returned over time.
- Add pagination for `GET /api/schedules`.
- Optional database credential/key rotation workflow for self-hosted deployments.
- Integration testing for backend (docker-compose with db)

## Non-Goals (for now)
- Multi-city support
- Authentication
- Payments
