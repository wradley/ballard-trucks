# User Data and Ops

## Scope
- Stored user data (favorites, saved filters, and related user preferences).
- Metrics and observability dashboard for API health and usage trends.
  - Track schedule query volume and schedules returned over time.
- Add pagination for `GET /api/schedules`.
- Optional database credential/key rotation workflow for self-hosted deployments.
- Integration testing for backend plus scraper persistence wiring in Docker.
- Add admin write APIs/UI for manually managing breweries/vendors with typed validation.
- Provide a simple operator command/script to export the Docker SQLite volume to a local file for inspection tooling (for example DataGrip).
