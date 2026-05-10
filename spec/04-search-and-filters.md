# Search and Filters

## Scope
- Lookup by food truck name.
- Refine `GET /api/schedules` with optional ID-based filters:
  - `brewery_ids` (comma-separated IDs)
  - `vendor_ids` (comma-separated IDs)
- Support narrow lookups (for example, all locations for a single vendor over a date window).
- Unknown `brewery_ids` / `vendor_ids` are valid and return zero matching rows (no error).
