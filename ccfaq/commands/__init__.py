"""
Commands provided by the ccfaq bot.
"""

from typing import Protocol, List, Optional

from prometheus_client import Summary

import discord
import discord.ext.commands as commands


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
