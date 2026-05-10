# Backend Foundation

## Scope
- [x] Hardcoded sample breweries: Stoup Brewing, Bale Breaker x Yonder, Urban Family.
  - Current state: seeded sample breweries are in place for Phase 1 development.
- [x] Sample schedule data is in place while scraper-to-storage integration is still in progress.
- [x] Read-only backend API only.
- [x] Request correlation IDs in logs (`X-Request-Id` propagation/generation) for per-request tracing.
- [x] Unit tests for Phase 1 API/domain behavior (including schedule query parameter validation).

## Data Model (Initial)
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
- vendor_id
- start_at (UTC RFC3339)
- end_at (UTC RFC3339)
- source (string)
- updated_at (RFC3339)

## API
- [x] GET /api/health -> "ok"
  - Current state: implemented, returns "ok".
- [x] GET /api/schedules?start_hour_utc=YYYY-MM-DDTHH&duration_hours=N -> ScheduleEntry[] (up to 100)
- [x] GET /api/breweries -> Brewery[]
- [x] GET /api/vendors -> Vendor[]

`GET /api/schedules` request contract (baseline):
- [x] `start_hour_utc` is a URL-encoded UTC hour bucket formatted as `YYYY-MM-DDTHH`.
- [x] `duration_hours` is an integer window size in hours.
- [x] API timestamps are UTC-only; client converts for local display.
  - Current state: schedule responses include UTC RFC3339 timestamps (`start_at`, `end_at`, `updated_at`).
- [x] Data updates regularly; schedule data more than 7 days ahead may be unavailable.
  - Current state: documentation guidance only (not server-enforced).
- [x] Response size is capped at 100 rows (pagination deferred to Phase 7).

## Deployment
- Build and run backend locally as a small Rust HTTP service.
- Confirm the binary runs in the local server environment.
- Keep Docker build paths aligned with local source layout (`/app/backend/...`) so compile-time includes (for example `include_str!` SQL files) resolve consistently in local and container builds.
- Reconcile SQLite schema on startup for additive column changes so existing volume data remains readable after backend updates.
- Emit structured startup diagnostics (including resolved SQLite path) and preserve full error chains in API logs to speed up DB incident debugging.
- Support a backend CLI SQLite override (`--sqlite-path <path>`) for local debugging against exported snapshots without changing container env defaults.
- Decode SQLite text UUID identifiers consistently across breweries, vendors, and schedules read paths.
- Keep DB repos table-scoped; compose cross-table ingest/supersession behavior in domain services.
- Cover ingest dedupe/supersession behavior with an integration test that runs against a real SQLite database file.
- For multi-arch Docker builds (local testing on Apple Silicon and x86_64 servers):
  - `docker buildx build --platform linux/arm64 -f backend/Dockerfile -t ballard-backend:arm64 .`
  - `docker buildx build --platform linux/amd64 -f backend/Dockerfile -t ballard-backend:amd64 .`
  - `docker run --rm ballard-backend:arm64`
- Optional later phase: set up a local Docker registry on the home network for fast LAN pulls.
- Local Docker test commands are documented in `README.md`.
