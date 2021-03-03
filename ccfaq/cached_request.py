"""
Provides a mechanism for querying a resource, then caching it for a
period of time.
"""

from typing import cast, TypeVar, Generic, Optional, Callable
from time import monotonic
from urllib.request import urlopen, Request
from urllib.error import HTTPError
import asyncio
import logging

LOG = logging.getLogger("cached_request")

T = TypeVar('T')  # pylint: disable=C0103

__all__ = ["CachedResource", "CachedRequest"]


class CachedResource(Generic[T]):
    """
    Caches a resource, preserving it for 'n' seconds before re-fetching it.
    """

    time_to_live: int

    _resource: Optional[T]
    _in_progress: Optional[asyncio.Task]
    _expire_at: float

    def __init__(self, ttl: int):
        self.time_to_live = ttl
        self._resource = None
        self._in_progress = None
        self._expire_at = monotonic()

    async def get(self) -> T:
        """Get the contents of this cache."""
        if self._expire_at >= monotonic() and self._resource is not None:
            return self._resource

        # If we've got a task running already, wait on that.
        if self._in_progress is not None:
            return await self._in_progress

        # Otherwise start a new task, wait on it, and then update the state
        self._in_progress = task = asyncio.create_task(self.fetch())
        try:
            self._resource = result = await task
        finally:
            self._in_progress = None
        self._expire_at = monotonic() + self.time_to_live

        return result

    async def fetch(self) -> T:
        """Recompute the contents of this cache."""
        raise NotImplementedError()


class CachedRequest(CachedResource[T]):
    """
    A HTTP request which is cached, and evaluates a function on the request body.
    """

    url: str
    _etag: Optional[str]

    def __init__(self, ttl: int, url: str, compute: Callable[[str], T]):
        super().__init__(ttl)
        self.url = url
        self.compute: Callable[[str], T] = compute
        self._etag = None

    async def fetch(self):
        def get() -> T:
            LOG.info("Fetching %s", self.url)
            request = Request(self.url)
            if self._etag is not None:
                request.add_header("If-None-Match", self._etag)

            try:
                with urlopen(request) as response:
                    self._etag = response.getheader("ETag")
                    contents = response.read()
                LOG.info("Finished request.")
                return self.compute(contents)
            except Exception as err:
                if isinstance(err, HTTPError) and err.code == 304:
                    LOG.info("ETag matched, doing nothing")
                    return cast(T, self._resource)

                LOG.exception("Failed to get resource.")
                if self._resource is not None:
                    return self._resource
                raise err

        return await asyncio.get_event_loop().run_in_executor(None, get)
