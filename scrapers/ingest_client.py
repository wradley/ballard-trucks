from dataclasses import dataclass

import aiohttp
from datetime import timezone

from models import Event


@dataclass(frozen=True)
class BreweryRecord:
    id: str
    name: str


class IngestClient:
    def __init__(self, *, base_url: str, api_key: str):
        if not base_url:
            raise RuntimeError("BACKEND_INGEST_URL is required")
        if not api_key:
            raise RuntimeError("INGEST_API_KEY is required")
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key

    async def ingest_events(
        self,
        *,
        run_id: str,
        source: str,
        brewery: BreweryRecord,
        events: list[Event],
    ) -> int:
        payload = {
            "run_id": run_id,
            "source": source,
            "brewery_id": brewery.id,
            "events": [
                {
                    "vendor_name": event.vendor.strip(),
                    "start_at": _event_time_rfc3339(event.start),
                    "end_at": _event_time_rfc3339(event.end),
                    "source_url": event.url,
                }
                for event in events
            ],
        }

        headers = {"Authorization": f"Bearer {self.api_key}"}
        async with aiohttp.ClientSession(headers=headers) as session:
            async with session.post(
                f"{self.base_url}/internal/ingest/schedules",
                json=payload,
            ) as response:
                body = await response.text()
                if response.status >= 400:
                    raise RuntimeError(
                        f"ingest failed ({response.status}): {body[:400]}"
                    )
                return len(events)


def _event_time_rfc3339(value):
    return (
        value.astimezone(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z")
    )
