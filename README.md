# ballard-trucks
Daily forecast of the local Ballard food trucks.

## Local Docker test (backend)
```sh
docker build -t ballard-backend-dev backend
docker run --rm -p 8080:8080 ballard-backend-dev
```

In another terminal:
```sh
curl http://localhost:8080/api/health
```

## Local Postgres (Docker)
```sh
docker compose up db
```

If you change major Postgres versions (for example, 16 -> 18), reset local data:
```sh
docker compose down -v
docker compose up db
```

Connect:
```sh
psql -h localhost -U ballard -d ballard_trucks
```

See `SPEC.md` for the current plan and phase breakdown.
See `openapi.yaml` for the current API contract (source of truth for request/response shape).

## High Level Design
![Architecture Diagram](/docs/trucks-arch.png)
