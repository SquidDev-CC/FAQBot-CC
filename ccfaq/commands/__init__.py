"""
Commands provided by the ccfaq bot.
"""

from __future__ import annotations

import functools
import logging
from typing import Any, Coroutine, Protocol, List, Optional, Callable, TypeVar, Awaitable, Union, cast
from typing_extensions import ParamSpec, Concatenate, TypeAlias

from opentelemetry.trace.status import Status, StatusCode
import opentelemetry.trace

from prometheus_client import Summary

import discord
from discord_slash.context import InteractionContext
import discord.ext.commands as commands


__all__ = ('Sendable', 'SendableContext', 'track_command')


log = logging.getLogger(__name__)
tracer = opentelemetry.trace.get_tracer(__name__)

class Sendable(Protocol):
    """A sink of messages."""
    async def send(
        self, *,
        content: str = "",
        embeds: Optional[List[discord.Embed]] = None,
        components: List[dict] = None,
    ) -> object:
        """Send a message with some optional content and optional embeds."""
        ...


class SendableContext:
    """Convert a standard context into a Sendable one."""

    def __init__(self, context: commands.Context):
        self.context = context

    async def send(self, *, content: str = "", embeds: Optional[List[discord.Embed]] = None, components: List[dict] = None) -> None:
        if embeds is None:
            embeds = []

        if len(embeds) == 0:
            await self.context.send(content=content, components=components)  # type: ignore
        elif len(embeds) == 1:
            await self.context.send(content=content, embed=embeds[0], components=components)  # type: ignore
        else:
            await self.context.send(content=content, components=components)  # type: ignore
            for embed in embeds:
                await self.context.send(embed=embed)


COMMAND_TIME = Summary("faqcc_command_time", "Time taken to execute a command", ["command", "mode"])


P = ParamSpec("P")
R = TypeVar("R")
C = TypeVar("C", bound=Union[InteractionContext, commands.Context])
Handler: TypeAlias = Callable[Concatenate[C, P], Coroutine[Any, Any, R]]

def track_command(command: str, mode: str) -> Callable[[Handler[C, P, R]], Handler[C, P, R]]:
    timer = COMMAND_TIME.labels(command, mode)
    op_name = f"{command}.{mode}"

    def create(func: Handler[C, P, R]) -> Handler[C, P, R]:
        @functools.wraps(func)
        async def wrapped(context: C, *args: P.args, **kwargs: P.kwargs) -> R:
            # Our types are a lie - we might have self as the first argument.
            main_context: C = cast(C, context if isinstance(context, (InteractionContext, commands.Context)) else args[0])

            with timer.time():
                with tracer.start_as_current_span(op_name) as span:
                    author, guild = main_context.author, main_context.guild
                    span.set_attribute("discord.user_name", author.display_name)
                    span.set_attribute("discord.user_tag", author.mention)
                    if guild is not None:
                        span.set_attribute("discord.guild", guild.name)

                    try:
                        log.info("Running %s (started by %s)", op_name, author.display_name)

                        return await func(context, *args, **kwargs)
                    except Exception as e:
                        span.set_status(Status(StatusCode.ERROR, str(e)))
                        span.record_exception(e)
                        raise e

        return wrapped
    return create
