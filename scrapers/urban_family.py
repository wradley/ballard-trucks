import json
import re
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

import aiohttp
from bs4 import BeautifulSoup

from models import Brewery, Event

_CALENDAR_URL = "https://urbanfamilybrewing.com/home/calendar/"
_AJAX_URL = "https://urbanfamilybrewing.com/wp-admin/admin-ajax.php"
_TZ = ZoneInfo("America/Los_Angeles")
# Sugar Calendar "Food Truck Calendar" — ID from page HTML: <input name="calendar-192">
_FOOD_TRUCK_CALENDAR_ID = 192
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
        next_start = datetime(year, month, 1, tzinfo=end.tzinfo)
        if next_start >= end:
            break


def _parse_events(html: str) -> list[tuple[str, datetime, datetime]]:
    """Return (vendor, start_utc, end_utc) tuples from a month-view HTML body."""
    soup = BeautifulSoup(html, "lxml")
    results = []

    for div in soup.find_all(attrs={"data-daydate": True}):
        # Secondary guard: skip events not on the Food Truck Calendar
        cal_info_raw = div.get("data-calendarsinfo")
        if cal_info_raw:
            try:
                cal_info = json.loads(cal_info_raw)
                cal_names = {c["name"] for c in cal_info.get("calendars", [])}
                if "Food Truck Calendar" not in cal_names:
                    continue
            except (json.JSONDecodeError, KeyError):
                pass

        title_el = div.find(class_="sugar-calendar-block__event-cell__title")
        time_el = div.find(class_="sugar-calendar-block__event-cell__time")

        if not title_el or not time_el:
            continue

        times = time_el.find_all("time")
        if len(times) < 2:
            continue

        vendor = title_el.get_text(strip=True)
        try:
            start_local = datetime.fromisoformat(times[0]["datetime"]).replace(tzinfo=_TZ)
            end_local = datetime.fromisoformat(times[1]["datetime"]).replace(tzinfo=_TZ)
        except (KeyError, ValueError) as exc:
            raise RuntimeError(
                f"Urban Family calendar time format has changed: {exc}"
            ) from exc

        results.append(
            (vendor, start_local.astimezone(timezone.utc), end_local.astimezone(timezone.utc))
        )

    return results


class UrbanFamilyBrewery(Brewery):
    def __init__(self):
        self._nonce: str | None = None

    async def _get_nonce(self, session: aiohttp.ClientSession) -> str:
        async with session.get(_CALENDAR_URL) as resp:
            html = await resp.text()
        match = re.search(r'"nonce":"([^"]+)"', html)
        if not match:
            raise RuntimeError(
                "Urban Family calendar page format has changed: nonce not found"
            )
        return match.group(1)

    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        start, end = range
        events: list[Event] = []

        async with aiohttp.ClientSession(trust_env=True, headers=_HEADERS) as session:
            nonce = await self._get_nonce(session)

            for year, month in _iter_months(start, end):
                data = {
                    "action": "sugar_calendar_block_update",
                    "nonce": nonce,
                    "calendar_block[sc_month]": month,
                    "calendar_block[sc_year]": year,
                    "calendar_block[sc_day]": 1,
                    "calendar_block[sc_display]": "month",
                    "calendar_block[sc_calendars]": _FOOD_TRUCK_CALENDAR_ID,
                }
                async with session.post(_AJAX_URL, data=data) as resp:
                    if resp.status != 200:
                        raise RuntimeError(
                            f"Urban Family AJAX returned {resp.status} for {year}-{month:02d}"
                        )
                    payload = await resp.json(content_type=None)

                if not payload.get("success"):
                    raise RuntimeError(
                        "Urban Family calendar AJAX request failed"
                    )

                try:
                    body_html = payload["data"]["body"]
                except KeyError as exc:
                    raise RuntimeError(
                        f"Urban Family AJAX response format has changed: {exc}"
                    ) from exc

                for vendor, ev_start, ev_end in _parse_events(body_html):
                    if ev_start <= end and ev_end >= start:
                        events.append(Event(vendor=vendor, start=ev_start, end=ev_end))

        return events
