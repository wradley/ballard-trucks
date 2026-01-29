# Ballard Trucks Backend

Forecast of the local Ballard food trucks.

## Backend Structure

- `backend/src/main.rs`: app bootstrap, router wiring, middleware wiring.
- `backend/src/api/`: HTTP handlers and request validation/extraction.
- `backend/src/domain/`: business/domain shaping from repo rows to API responses.
- `backend/src/db/`: SQLx row types and repository traits/implementations.
- `backend/src/middleware.rs`: cross-cutting HTTP middleware (`x-request-id`).
