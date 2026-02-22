from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime


@dataclass
class Event:
    vendor: str
    start: datetime       # UTC-aware
    end: datetime         # UTC-aware
    url: str | None = None
    estimated_times: bool = False


class Brewery(ABC):
    @abstractmethod
    async def get_dates(self, range: tuple[datetime, datetime]) -> list[Event]:
        """Return events within [range[0], range[1]] (UTC-aware datetimes).

        Raises RuntimeError if the source format has changed and cannot be parsed.
        """
        ...
