# Schedule Scraping

## Scope
- Cached schedule persistence is required, but the storage implementation is still being simplified.
- Deployment test to local server environment.
- Add scheduled daily scraping + on-request refresh queue.
- Replace “harvesters” with Python scrapers that write directly to persistent backend storage.
- Run Python scrapers in Docker with pinned dependencies for reproducible execution.
- Scrapers ingest through a backend-owned internal write endpoint so SQLite write logic and cleanup rules are centralized.
- Backend exclusively owns SQLite file access; scrapers do not mount or access the SQLite volume directly.
- Resolve schema/seed SQL file locations relative to the scraper application directory so local and Docker execution use the same initialization paths.

## Storage Direction
- Chosen direction: SQLite on a shared local volume used by both containers.
- Reasoning:
  - keeps the “single local file” deployment model,
  - avoids running a separate database container,
  - supports indexed reads, filtering, idempotent upserts, and safer concurrency between scraper writes and API reads.
- Raw file storage is rejected for now because it pushes locking, atomic updates, and query behavior into application code.
- Avoid defaulting to live runtime scraping inside API requests.
  - Reasoning: it makes API latency and correctness depend on third-party sites, increases breakage risk, and removes the value of a stable cache.
- A later manual refresh path is still acceptable, but it should refresh cached data asynchronously rather than block a user-facing read request.

## Scraper Data Flow
- Python scrapers extract raw events with:
  - `source`
  - `brewery_id`
  - `vendor_name`
  - `start_at`
  - `end_at`
  - optional source URL and scrape timestamp
- Backend normalizes vendor names during ingest:
  - lowercase,
  - trim surrounding whitespace,
  - collapse repeated internal whitespace,
  - remove punctuation and separator noise.
- Backend resolves vendor identity before writing a schedule row:
  - exact match on `vendors.normalized_name` first,
  - conservative fuzzy/variant match against existing normalized names for common suffixes and near-typos,
  - otherwise create a new vendor row and mark it `needs_review = true`.
- Schedule rows are written with `vendor_id` so the backend can serve a clean canonical model.

### Initial Implementation Slice
- First persistence pass writes scraper output directly into SQLite from Python.
- Brewery identity is resolved from a static scraper-side mapping for known breweries.
- Vendor identity resolution behavior for this slice:
  - exact normalized-name match first,
  - then conservative fuzzy matching in Python against existing vendor rows,
  - otherwise create a new vendor row with `needs_review = true`.
- Schedule persistence behavior for this slice:
  - write canonical `vendor_id`,
  - use idempotent upsert on the deterministic source identity key,
  - defer full snapshot supersession / stale-row cleanup until a later step.

## Vendor Resolution
- Vendor matching is scraper-side logic implemented in Python rather than SQLite-native fuzzy search.
- Keep both raw and canonical names:
  - `vendors.name` is the canonical display name.
  - `vendors.normalized_name` is the lookup key.
- Matching strategy:
  - exact normalized-name match,
  - high-threshold fuzzy match against existing normalized names,
  - else create a new vendor.
- Review behavior:
  - exact and trusted fuzzy matches can remain `needs_review = false`,
  - newly created vendors default to `needs_review = true`,
  - borderline fuzzy matches may also be flagged for review if the threshold policy is conservative.
- Persist match provenance metadata where useful:
  - `match_method` such as `exact`, `fuzzy`, or `created`,
  - `match_score` when fuzzy matching is used.

## Persistence and Corrections
- Deterministic key: `source + brewery_id + vendor_id + start_at + end_at`.
- Scrapers use direct upsert semantics for identical keys.
- Same-day correction behavior:
  - Backend marks prior active entries for that source+brewery as superseded when a newer snapshot run arrives.
  - Read APIs should prefer latest active snapshot for a source/day to avoid mixed stale/current rows.
  - UI should be able to surface freshness/superseded state to end users when schedule changes are detected.

### Backend Ingest API (Internal)
- `POST /internal/ingest/schedules` accepts a full scraped batch for one `source` + `brewery`.
- Request includes:
  - `run_id` (string)
  - `source` (string)
  - `brewery_id`
  - `events[]` containing `vendor_name`, `start_at`, `end_at`, optional `source_url`
- Authentication:
  - Bearer token in `Authorization` header (`INGEST_API_KEY`).
- Behavior:
  - Upsert each event by deterministic identity key,
  - mark prior active future rows for the same source+brewery not in the current run as superseded,
  - delete rows older than retention window (`end_at < now - retention_days`).

## Data Model
Vendor:
- id (string)
- name
- normalized_name
- website (optional)
- needs_review (boolean)
- match_method (optional)
- match_score (optional)

ScheduleEntry:
- id (string)
- source
- brewery_id
- vendor_id
- start_at
- end_at
- source_url (optional)
- scraped_at
- updated_at

## Scraping Strategy
- Prefer APIs / calendar feeds / JSON-LD over HTML parsing.
- Cache daily snapshots and keep metadata (source URL, last-modified).
- Rate limit and respect robots.txt.

## Refresh Policy
- Store `last_scraped_at` and `refresh_requested_at` in SQLite.
- Backend sets `refresh_requested_at = now()` only if:
  - data is stale beyond a `min_stale` threshold, and
  - `now - refresh_requested_at` exceeds a `min_request_interval` (dedupe).
- Scraper runner runs on a fixed schedule (e.g., twice/day or every 15 minutes) and:
  - ensures only one scraper runner is actively writing at a time,
  - checks if a refresh is requested or scheduled,
  - scrapes, updates `last_scraped_at`, and clears `refresh_requested_at`.

## Optional Experiment: LLM-Assisted HTML Extraction
- Timebox: one afternoon.
- Goal: measure whether a local LLM can reliably extract food vendor schedules from rendered brewery pages.
- Proposed flow:
  - Render the page with a headless browser and capture the final HTML snapshot.
  - Prompt a local LLM to return strict JSON schedule candidates.
  - Ask/score whether each candidate is a food vendor (skip non-food events).
  - Validate output schema and timestamps before writing directly to persistent storage.
- Safety and quality constraints:
  - Keep deterministic validation before persistence (required fields, UTC timestamps, dedupe key checks).
  - Track confidence and evidence text per extracted item.
  - Do not auto-ingest low-confidence items.
- Success criteria:
  - Compare extracted output against a small labeled sample set.
  - Record precision/recall and timestamp accuracy.
  - Decide whether to keep LLM parsing as fallback-only or promote for specific sources.
