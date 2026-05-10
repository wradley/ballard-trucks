from datetime import datetime, timezone
from html import unescape

import aiohttp

from models import Brewery, Event

_API_URL = "https://www.bbycballard.com/api/open/GetItemsByMonth"
_COLLECTION_ID = "61328af17400707612fccbc6"
_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
}


def _iter_months(start: datetime, end: datetime):
    """Yield (year, month) pairs that overlap with the half-open interval [start, end)."""
    year, month = start.year, start.month
    while True:
        yield year, month
        month += 1
        if month > 12:
            month = 1
            year += 1
        if (year, month) > (end.year, end.month):
            break
        # Don't fetch a month that starts exactly at or after end
        next_start = datetime(year, month, 1, tzinfo=end.tzinfo)
        if next_start >= end:
            break


class BbycBallardBrewery(Brewery):
    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        start, end = range
        events: list[Event] = []

        async with aiohttp.ClientSession(trust_env=True, headers=_HEADERS) as session:
            for year, month in _iter_months(start, end):
                async with session.get(
                    _API_URL,
                    params={"month": f"{month:02d}-{year}", "collectionId": _COLLECTION_ID},
                ) as resp:
                    if resp.status != 200:
                        raise RuntimeError(
                            f"BBYC Ballard API returned {resp.status} for {year}-{month:02d}"
                        )
                    items = await resp.json(content_type=None)

                if not isinstance(items, list):
                    raise RuntimeError(
                        "BBYC Ballard API response format has changed: expected a list"
                    )

                for item in items:
                    try:
                        event_start = datetime.fromtimestamp(
                            item["startDate"] / 1000, tz=timezone.utc
                        )
                        event_end = datetime.fromtimestamp(
                            item["endDate"] / 1000, tz=timezone.utc
                        )
                        vendor = unescape(item["title"])
                    except (KeyError, TypeError) as exc:
                        raise RuntimeError(
                            f"BBYC Ballard API item format has changed: {exc}"
                        ) from exc

                    if event_start <= end and event_end >= start:
                        events.append(Event(vendor=vendor, start=event_start, end=event_end))

        return events
