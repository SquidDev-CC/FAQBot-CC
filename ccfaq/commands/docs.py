"""Searches the CC:Tweaked documentation."""

from typing import Iterable, Tuple, Callable
from difflib import SequenceMatcher
import asyncio
import logging
import json

import discord
import discord.ext.commands as commands
from discord_slash.cog_ext import cog_slash
from discord_slash import SlashContext, SlashCommand
import discord_slash.utils.manage_commands as manage_commands

from ccfaq.cached_request import CachedRequest
from ccfaq.commands import Sendable, SendableContext, COMMAND_TIME
from ccfaq.config import guild_ids
from ccfaq.lua_names import NAMES as lua_names
from ccfaq.timing import with_async_timer


LOG = logging.getLogger(__name__)


def _score_methods(methods: Iterable[str], search: str, threshold: float = 0.8) -> Iterable[Tuple[str, float]]:
    """
    Filter a set of methods for those which match a search string with a "good
    enough" score.
    """
    for method in methods:
        score = SequenceMatcher(None, method, search).ratio()
        if score >= threshold:
            yield method, score


def _embed(method: dict, link: Callable[[dict], str]) -> discord.Embed:
    """Generate an embed linking to a particular method."""
    embed = discord.Embed(title=method["name"], url=link(method))
    if "summary" in method:
        embed.description = method["summary"]
    return embed


class DocsCog(commands.Cog):
    """
    Provides a %doc (%d) and %source (%s) command.
    """

    def __init__(self):
        self.methods = CachedRequest(
            60, "https://tweaked.cc/index.json",
            lambda contents: {
                k.lower(): {"original_name": k, **v}
                for k, v in json.loads(contents).items()
            }
        )

    async def _search_docs(self, ctx: Sendable, search: str, link: Callable[[dict], str]) -> None:
        """Search the documentation with a query and link to the result"""
        methods = await self.methods.get()

        search_k = search.lower()
        if search_k in methods:
            LOG.info(f'event=search search="{search}"')
            await ctx.send(embeds=[_embed(methods[search_k], link)])
            return

        # We've not found a perfect match, so find an approximate one. A "good" match
        # is either a unique one, or one with a significantly higher score than the
        # next best one.
        best_matches = sorted(
            _score_methods(methods, search_k),
            key=lambda k: k[1], reverse=True,
        )
        if (
            len(best_matches) == 1 or
            (len(best_matches) >=
             2 and best_matches[0][1] >= best_matches[1][1] + 0.05)
        ):
            best_match, _ = best_matches[0]
            method = methods[best_match]
            LOG.info(f'event=search.approx search="{search}" result="{method["original_name"]}"')
            await ctx.send(
                content=f"Cannot find '{search}', using '{method['original_name']}'' instead.",
                embeds=[_embed(method, link)],
            )
            return

        if search_k in lua_names:
            LOG.info(f'event=search.lua search="{search}"')
            link = lua_names[search_k]
            await ctx.send(embeds=[discord.Embed(title=search_k, url=link)])
            return

        LOG.warning(f'event=search.missing search="{search}"')
        await ctx.send(content=f"Cannot find method '{search}'. Please check your spelling, or contribute to the documentation at https://github.com/SquidDev-CC/CC-Tweaked.")

    @commands.command(name="doc", aliases=["d", "docs"])
    @with_async_timer(COMMAND_TIME.labels('doc', 'message'))
    async def doc(self, ctx: commands.Context, *, search: str) -> None:
        """Searches for a function with the current name and returns its documentation."""
        await self._search_docs(SendableContext(ctx), search, lambda x: f"https://tweaked.cc/{x['url']}")

    @commands.command(name="source", aliases=["s"])
    @with_async_timer(COMMAND_TIME.labels('source', 'message'))
    async def source(self, ctx: commands.Context, *, search: str) -> None:
        """Searches for a function with the current name, and returns a link to its source code."""
        await self._search_docs(SendableContext(ctx), search, lambda x: x["source"])

    @doc.error
    @source.error
    async def doc_error(self, ctx: commands.Context, error) -> None:
        """Reports an error on the source and doc commands."""
        if isinstance(error, commands.MissingRequiredArgument):
            await ctx.send(content="Missing arguments! Please provide a CC:T method to search for.")
        else:
            LOG.exception("Error processing command: %s", error)
            await ctx.send("An unexpected error occurred when processing the command.")

    @cog_slash(
        name="docs", description="Searches for a function with the current name and returns its documentation.",
        options=[
            manage_commands.create_option(
                name="name",
                description="The function's name",
                option_type=3,
                required=True,
            ),
        ],
        guild_ids=guild_ids(),
    )
    @with_async_timer(COMMAND_TIME.labels('doc', 'slash'))
    async def doc_slash(self, ctx: SlashContext, name: str) -> None:
        await self._search_docs(ctx, name, lambda x: f"https://tweaked.cc/{x['url']}")

    @cog_slash(
        name="source", description="Searches for a function with the current name and returns a link to its source code.",
        options=[
            manage_commands.create_option(
                name="name",
                description="The function's name",
                option_type=3,
                required=True,
            ),
        ],
        guild_ids=guild_ids(),
    )
    @with_async_timer(COMMAND_TIME.labels('source', 'slash'))
    async def doc_source(self, ctx: SlashContext, name: str) -> None:
        await self._search_docs(ctx, name, lambda x: x["source"])

    def cog_unload(self) -> None:
        asyncio.ensure_future(self.methods.close())
