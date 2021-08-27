import logging
import logging
from timeit import default_timer
import functools

import aiohttp
import wrapt
from prometheus_client import Summary # Ideally we'd use opentelemetry here, but I can't find the prom exporter.
from opentelemetry.trace import set_tracer_provider, get_current_span, format_span_id, format_trace_id
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, ConsoleSpanExporter
from opentelemetry.propagate import set_global_textmap
from opentelemetry.propagators.jaeger import JaegerPropagator
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.aiohttp_client import AioHttpClientInstrumentor

from .config import metrics_port


__all__ = ('configure', 'TraceFilter', 'with_async_timer')


def with_async_timer(stat: Summary):
    """Record how long a function takes to a Summary."""
    def create(func):
        @functools.wraps(func)
        async def wrapped(*args, **kwargs):
            # Obtaining new instance of timer every time
            # ensures thread safety and reentrancy.
            with stat.time():
                return await func(*args, **kwargs)

        return wrapped

    return create


def _aiohttp_add_timings():
    """Instrument various aiohttp events with timing information."""
    trace_config = aiohttp.TraceConfig(trace_config_ctx_factory=dict)
    aiohttp_time = Summary("faqcc_aiohttp_time", "Time for a given aiohttp operation", ['op'])

    def _setup_trace_config(field: str) -> None:
        summary = aiohttp_time.labels(field)

        async def start(session, ctx, params) -> None:
            ctx[field] = default_timer()

        async def end(session, ctx, params) -> None:
            summary.observe(default_timer() - ctx[field])

        getattr(trace_config, f"{field}_start").append(start)
        getattr(trace_config, f"{field}_end").append(end)


    for field in ('on_request', 'on_connection_queued', 'on_connection_create', 'on_dns_resolvehost'):
        _setup_trace_config(field)

    def wrapped_init(wrapped, _instance, args, kwargs):
        trace_configs = list(kwargs.get("trace_configs") or ())
        trace_configs.append(trace_config)
        kwargs["trace_configs"] = trace_configs
        wrapped(*args, **kwargs)


    wrapt.wrap_function_wrapper(aiohttp.ClientSession, "__init__", wrapped_init)


class TraceFilter(logging.Filter):
    def filter(self, record) -> bool:
        context = get_current_span().get_span_context()
        if context.is_valid:
            record.trace_id = format_trace_id(context.trace_id)
            record.span_id = format_span_id(context.span_id)
        else:
            record.trace_id = "-"
            record.span_id = "-"
        return True


def configure():
    """
    Configure OpenTelemetry metrics.
    """
    set_global_textmap(JaegerPropagator())

    resource = Resource(attributes={
        "service.name": "faq-cc"
    })

    span_exporter = OTLPSpanExporter() if metrics_port () is not None else ConsoleSpanExporter()
    span_processor = BatchSpanProcessor(span_exporter)

    provider = TracerProvider(resource=resource)
    set_tracer_provider(provider)
    provider.add_span_processor(span_processor)

    _aiohttp_add_timings()
    AioHttpClientInstrumentor().instrument()
