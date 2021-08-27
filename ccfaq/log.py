"""Helper methods to configure the system logger."""
import asyncio
import logging
import logging.config
import sys
from typing import Any, Dict
import warnings

from .telemetry import TraceFilter


BLACK, RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, WHITE = range(8)

COLOURS = {
    'WARNING': YELLOW,
    'INFO': WHITE,
    'DEBUG': BLUE,
    'CRITICAL': YELLOW,
    'ERROR': RED
}


def loop_exception_handler(loop: asyncio.AbstractEventLoop, context: Dict[str, Any]) -> None:
    """A custom error handler for the loop, which stops the loop before continuing to
       the default handler

    """
    logging.error("Terminating loop due to error")
    if loop.is_running():
        loop.stop()
    loop.default_exception_handler(context)


class ColourFormatter(logging.Formatter):
    """Formats log messages using ANSI escape codes."""

    def format(self, record: logging.LogRecord) -> str:
        msg = super().format(record)
        if record.levelname in COLOURS:
            msg = "\033[1;3%dm%s\033[0m" % (COLOURS[record.levelname], msg)
        return msg


FORMAT = "[%(asctime)s] level=%(levelname)s logger=%(name)s trace_id=%(trace_id)s span_id=%(span_id)s %(message)s"


def configure() -> None:
    """
    Configure the root logger. This should be called once when the program is
    initialised.
    """
    # Be more aggressive in capturing warnings
    logging.captureWarnings(True)
    warnings.simplefilter('default')

    # Register a custom formatter, which prints things coloured with the time, level and coponent
    # name.
    formatter: logging.Formatter
    if sys.stdout.isatty() and sys.stderr.isatty():
        formatter = ColourFormatter(FORMAT, None, '%')
    else:
        formatter = logging.Formatter(FORMAT, None, '%')
    formatter.default_msec_format = "%s.%03d"

    str_handler = logging.StreamHandler()
    str_handler.addFilter(TraceFilter())
    str_handler.setFormatter(formatter)
    str_handler.setLevel(logging.INFO)

    logging.basicConfig(level=logging.DEBUG, handlers=[str_handler])
