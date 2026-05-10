import re
from datetime import date, datetime, time, timezone
from zoneinfo import ZoneInfo

import aiohttp
from bs4 import BeautifulSoup

from models import Brewery, Event

_EVENTS_URL = "https://fairislebrewing.com/events/"
_BASE_URL = "https://fairislebrewing.com"
_TZ = ZoneInfo("America/Los_Angeles")
_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
}

# La Marea hardcoded schedule (local Pacific time)
_WEEKDAY_START = time(17, 0)   # 5:00 PM
_WEEKDAY_END = time(20, 0)     # 8:00 PM
_WEEKEND_START = time(13, 0)   # 1:00 PM
_WEEKEND_END = time(19, 0)     # 7:00 PM

_DATE_RE = re.compile(r"/event/.+-(\d{4}-\d{2}-\d{2})/?$")
_FOOD_CATEGORIES = {"kitchen", "pop-up", "pop up"}


def _is_food_event(category_text: str) -> bool:
    lower = category_text.lower()
    return any(kw in lower for kw in _FOOD_CATEGORIES)


def _event_times(d: date) -> tuple[datetime, datetime]:
    is_weekend = d.weekday() >= 5  # Saturday=5, Sunday=6
    start_t = _WEEKEND_START if is_weekend else _WEEKDAY_START
    end_t = _WEEKEND_END if is_weekend else _WEEKDAY_END
    start = datetime.combine(d, start_t, tzinfo=_TZ).astimezone(timezone.utc)
    end = datetime.combine(d, end_t, tzinfo=_TZ).astimezone(timezone.utc)
    return start, end


class FairIsleBrewery(Brewery):
    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        start, end = range
        events: list[Event] = []

        async with aiohttp.ClientSession(trust_env=True, headers=_HEADERS) as session:
            async with session.get(_EVENTS_URL) as resp:
                if resp.status != 200:
                    raise RuntimeError(
                        f"Fair Isle events page returned {resp.status}"
                    )
                html = await resp.text()

        soup = BeautifulSoup(html, "lxml")
        links = soup.find_all("a", href=_DATE_RE)

        if not links:
            raise RuntimeError(
                "Fair Isle events page format has changed: no event links with date slugs found"
            )

        seen_urls: set[str] = set()
        for link in links:
            href = link.get("href", "")
            if href in seen_urls:
                continue
            seen_urls.add(href)

            m = _DATE_RE.search(href)
            if not m:
                continue

            event_date = date.fromisoformat(m.group(1))

            # Filter by category — BeautifulSoup passes each class string individually
            category_el = link.find(class_=lambda c: c and "ui-tag" in c)
            if category_el:
                if not _is_food_event(category_el.get_text(strip=True)):
                    continue

            title_el = link.find(class_=lambda c: c and "excerpt-box-title" in c)
            vendor = title_el.get_text(strip=True) if title_el else link.get_text(strip=True)

            ev_start, ev_end = _event_times(event_date)
            url = _BASE_URL + href if href.startswith("/") else href

            if ev_start <= end and ev_end >= start:
                events.append(
                    Event(vendor=vendor, start=ev_start, end=ev_end, url=url, estimated_times=True)
                )

        return events
