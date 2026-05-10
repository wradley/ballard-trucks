import html
import json
import re
from datetime import date, datetime, time, timezone
from zoneinfo import ZoneInfo

import aiohttp
from bs4 import BeautifulSoup

from models import Brewery, Event

_EVENTS_URL = "https://www.luckyenvelopebrewing.com/events"
_TZ = ZoneInfo("America/Los_Angeles")
_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
}

# Matches: "Fri 3/13/26 5pm-8pm" or "Sat 3/28/26 4:30-7:30pm"
_ENTRY_RE = re.compile(
    r"\w+\s+(\d{1,2})/(\d{1,2})/(\d{2,4})\s+"
    r"(\d+(?::\d+)?)(?:[ap]m)?\s*-\s*(\d+(?::\d+)?)[ap]m",
    re.IGNORECASE,
)


def _parse_hm(s: str) -> time:
    """Parse '4' or '4:30' as a PM time (all Lucky Envelope trucks are afternoon)."""
    if ":" in s:
        h, m = int(s.split(":")[0]), int(s.split(":")[1])
    else:
        h, m = int(s), 0
    if h < 12:
        h += 12
    return time(h, m)


class LuckyEnvelopeBrewery(Brewery):
    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        start, end = range
        events: list[Event] = []

        async with aiohttp.ClientSession(trust_env=True, headers=_HEADERS) as session:
            async with session.get(_EVENTS_URL) as resp:
                if resp.status != 200:
                    raise RuntimeError(
                        f"Lucky Envelope events page returned {resp.status}"
                    )
                page_html = await resp.text()

        soup = BeautifulSoup(page_html, "lxml")
        carousel = soup.find("div", class_="user-items-list-carousel")
        if not carousel:
            raise RuntimeError(
                "Lucky Envelope events page format has changed: food truck carousel not found"
            )

        try:
            ctx = json.loads(carousel.get("data-current-context", "{}"))
        except json.JSONDecodeError as exc:
            raise RuntimeError(
                f"Lucky Envelope carousel JSON invalid: {exc}"
            ) from exc

        user_items = ctx.get("userItems", [])
        if not user_items:
            raise RuntimeError(
                "Lucky Envelope events page format has changed: no items in food truck carousel"
            )

        for item in user_items:
            vendor = item.get("title", "").strip()
            desc_html = html.unescape(item.get("description", ""))
            desc_text = BeautifulSoup(desc_html, "lxml").get_text(" ", strip=True)

            m = _ENTRY_RE.search(desc_text)
            if not m:
                continue

            month, day = int(m.group(1)), int(m.group(2))
            year_str = m.group(3)
            year = int(year_str) + (2000 if len(year_str) == 2 else 0)
            ev_date = date(year, month, day)

            ev_start = datetime.combine(ev_date, _parse_hm(m.group(4)), tzinfo=_TZ).astimezone(timezone.utc)
            ev_end = datetime.combine(ev_date, _parse_hm(m.group(5)), tzinfo=_TZ).astimezone(timezone.utc)

            desc_soup = BeautifulSoup(desc_html, "lxml")
            url_el = desc_soup.find("a")
            url = url_el.get("href") if url_el else None

            if ev_start <= end and ev_end >= start:
                events.append(Event(vendor=vendor, start=ev_start, end=ev_end, url=url))

        return events
