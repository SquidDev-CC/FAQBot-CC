"""
Provides a mechanism for querying a resource, then caching it for a
period of time.
"""

from typing import cast, TypeVar, Generic, Optional, Callable
from time import monotonic
import asyncio
import aiohttp
import logging

from prometheus_client import Summary

from .utils import with_async_timer


LOG = logging.getLogger("cached_request")

T = TypeVar('T')  # pylint: disable=C0103

__all__ = ["CachedResource", "CachedRequest"]


class CachedResource(Generic[T]):
    """
    Caches a resource, preserving it for 'n' seconds before re-fetching it.
    """

    def __init__(self, ttl: int):
        self.time_to_live = ttl
        self._resource: Optional[T] = None
        self._in_progress: Optional[asyncio.Task] = None
        self._expire_at: float = monotonic()

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


REQUEST_TIME = Summary('faqcc_cached_resource_request', 'Request time of cached resources')


class CachedRequest(CachedResource[T]):
    """
    A HTTP request which is cached, and evaluates a function on the request body.
    """

    def __init__(self, ttl: int, url: str, compute: Callable[[str], T]):
        super().__init__(ttl)
        self.url = url
        self.compute: Callable[[str], T] = compute
        self._etag: Optional[str] = None
        self._session = aiohttp.ClientSession()

    @with_async_timer(REQUEST_TIME)
    async def fetch(self):
        LOG.info("Fetching %s", self.url)

        headers = {}
        if self._etag is not None:
            headers["If-None-Match"] = self._etag

        start = monotonic()
        try:
            async with self._session.get(self.url, headers=headers) as response:
                if response.status == 304:
                    LOG.info("ETag matched, doing nothing")
                    return cast(T, self._resource)
                elif response.status == 200:
                    self._etag = response.headers.get("ETag")
                    contents = await response.text()
                else:
                    raise Exception(f"Error getting resource ({response.status} {response.reason})")

            return self.compute(contents)

        except Exception as err:
            LOG.exception("Failed to get resource.")
            if self._resource is not None:
                return self._resource
            raise err

    async def close(self) -> None:
        self._session.close()
