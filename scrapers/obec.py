import re
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

import aiohttp
from bs4 import BeautifulSoup

from models import Brewery, Event

_HOME_URL = "https://obecbrewing.com/"
_TZ = ZoneInfo("America/Los_Angeles")
_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
}

_TRUCK_RE = re.compile(
    r"Food\s+truck:\s*(.+?)\s+(\d{1,2}:\d{2})\s*-\s*(\d{1,2}:\d{2})",
    re.IGNORECASE,
)


def _to_pm_hour(h: int) -> int:
    """Convert a bare hour to 24-hour PM time (brewery hours are always afternoon)."""
    return h + 12 if h < 12 else h


class ObecBrewery(Brewery):
    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        start, end = range

        today_local = datetime.now(tz=_TZ).date()
        today_start = datetime(today_local.year, today_local.month, today_local.day,
                               tzinfo=_TZ).astimezone(timezone.utc)
        today_end = today_start.replace(hour=23, minute=59, second=59)

        # Only return data if today overlaps the requested range
        if today_start > end or today_end < start:
            return []

        async with aiohttp.ClientSession(trust_env=True, headers=_HEADERS) as session:
            async with session.get(_HOME_URL) as resp:
                if resp.status != 200:
                    raise RuntimeError(f"Obec homepage returned {resp.status}")
                html = await resp.text()

        soup = BeautifulSoup(html, "lxml")

        for span in soup.find_all("span", class_="btIconWidgetTitle"):
            text = span.get_text(strip=True)
            m = _TRUCK_RE.search(text)
            if not m:
                if "food truck" in text.lower():
                    raise RuntimeError(
                        f"Obec food truck text format has changed: {text!r}"
                    )
                continue

            vendor = m.group(1).strip()
            start_h, start_min = map(int, m.group(2).split(":"))
            end_h, end_min = map(int, m.group(3).split(":"))

            ev_start = datetime(
                today_local.year, today_local.month, today_local.day,
                _to_pm_hour(start_h), start_min, tzinfo=_TZ
            ).astimezone(timezone.utc)
            ev_end = datetime(
                today_local.year, today_local.month, today_local.day,
                _to_pm_hour(end_h), end_min, tzinfo=_TZ
            ).astimezone(timezone.utc)

            return [Event(vendor=vendor, start=ev_start, end=ev_end)]

        return []
