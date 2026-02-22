import asyncio
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from uuid import uuid4

from ingest_client import BreweryRecord, IngestClient
from bbyc_ballard import BbycBallardBrewery
from fair_isle import FairIsleBrewery
from lucky_envelope import LuckyEnvelopeBrewery
from models import Brewery
from obec import ObecBrewery
from urban_family import UrbanFamilyBrewery


@dataclass(frozen=True)
class ScraperConfig:
    label: str
    source: str
    brewery: BreweryRecord
    scraper: type[Brewery]


async def main():
    now = datetime.now(tz=timezone.utc)
    month_start = now.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
    if month_start.month == 12:
        month_end = month_start.replace(year=month_start.year + 1, month=1)
    else:
        month_end = month_start.replace(month=month_start.month + 1)

    date_range = (month_start, month_end)
    print(f"Fetching events from {month_start.date()} to {month_end.date()}\n")

    scrapers = [
        ScraperConfig(
            label="BBYC Ballard",
            source="bbyc-ballard",
            brewery=BreweryRecord(
                id="c8176998-6c38-4813-a9ec-1e45a710e6dc",
                name="Bale Breaker x Yonder Cider",
            ),
            scraper=BbycBallardBrewery,
        ),
        ScraperConfig(
            label="Urban Family",
            source="urban-family",
            brewery=BreweryRecord(
                id="64b2ec8a-41ff-44c8-9b91-b0d7c3e0457b",
                name="Urban Family Brewing Co.",
            ),
            scraper=UrbanFamilyBrewery,
        ),
        ScraperConfig(
            label="Lucky Envelope",
            source="lucky-envelope",
            brewery=BreweryRecord(
                id="f5a98880-fd2d-4c25-b72d-0a7f89fcd866",
                name="Lucky Envelope Brewing",
            ),
            scraper=LuckyEnvelopeBrewery,
        ),
        ScraperConfig(
            label="Fair Isle",
            source="fair-isle",
            brewery=BreweryRecord(
                id="6373ac59-7f83-4565-8f94-9d4d6d7582ec",
                name="Fair Isle Brewing",
            ),
            scraper=FairIsleBrewery,
        ),
        ScraperConfig(
            label="Obec",
            source="obec",
            brewery=BreweryRecord(
                id="0f4f2392-dc38-493f-aed3-9dfba21d51b0",
                name="Obec Brewing",
            ),
            scraper=ObecBrewery,
        ),
    ]

    ingest_client = IngestClient(
        base_url=os.environ.get("BACKEND_INGEST_URL", "http://backend:8080"),
        api_key=os.environ.get("INGEST_API_KEY", ""),
    )
    run_id = str(uuid4())

    for config in scrapers:
        print(f"=== {config.label} ===")
        try:
            events = await config.scraper().get_dates(date_range)
            sorted_events = sorted(events, key=lambda event: event.start)
            if not sorted_events:
                print("  (no events)")
            for event in sorted_events:
                est = " (estimated)" if event.estimated_times else ""
                url = f"  {event.url}" if event.url else ""
                print(
                    f"  {event.start.strftime('%Y-%m-%d %H:%M UTC')} – "
                    f"{event.end.strftime('%H:%M UTC')}  {event.vendor}{est}{url}"
                )

            persisted = await ingest_client.ingest_events(
                run_id=run_id,
                source=config.source,
                brewery=config.brewery,
                events=sorted_events,
            )
            print(f"  ingested {persisted} event(s) through backend API")
        except Exception as exc:
            print(f"  ERROR: {exc}")
        print()


if __name__ == "__main__":
    asyncio.run(main())
