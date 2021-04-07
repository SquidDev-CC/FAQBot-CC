import functools
import aiohttp
from timeit import default_timer
from prometheus_client import Summary

__all__ = ('with_async_timer', 'TRACE_CONFIG')

def with_async_timer(stat):
    def create(func):
        @functools.wraps(func)
        async def wrapped(*args, **kwargs):
            # Obtaining new instance of timer every time
            # ensures thread safety and reentrancy.
            with stat.time():
                return await func(*args, **kwargs)

        return wrapped

    return create


TRACE_CONFIG = aiohttp.TraceConfig(trace_config_ctx_factory=dict)

_AIOHTTP_TIME = Summary("faqcc_aiohttp_time", "Time for a given aiohttp operation", ['op'])

def _setup_trace_config(field: str) -> None:
    summary = _AIOHTTP_TIME.labels(field)

    async def start(session, ctx, params) -> None:
        ctx[field] = default_timer()

    async def end(session, ctx, params) -> None:
        summary.observe(default_timer() - ctx[field])

    getattr(TRACE_CONFIG, f"{field}_start").append(start)
    getattr(TRACE_CONFIG, f"{field}_end").append(end)


for field in ('on_request', 'on_connection_queued', 'on_connection_create', 'on_dns_resolvehost'):
    _setup_trace_config(field)


class _ClientSession(aiohttp.ClientSession):
    """Ugly monkey patch to instrument all HTTP requests."""
    def __init__(self, *args, **kwargs):
        trace_configs = kwargs.get('trace_configs', None)
        if trace_configs is None:
            kwargs['trace_configs'] = [TRACE_CONFIG]

        super().__init__(*args, **kwargs)


aiohttp.ClientSession = _ClientSession
