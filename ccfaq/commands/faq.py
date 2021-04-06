"""Provides commands to read FAQs"""
from typing import List
import logging
import re

import discord
import discord.ext.commands as commands
from discord_slash import SlashCommand, SlashContext
import discord_slash.utils.manage_commands as manage_commands

from ccfaq.commands import Sendable, SendableContext, COMMAND_TIME
from ccfaq.config import guild_ids
from ccfaq.faq_list import FAQ
from ccfaq.utils import with_async_timer


LOG = logging.getLogger(__name__)


def _embed(faq: FAQ) -> discord.Embed:
    return discord.Embed(title=faq.title, colour=discord.Colour(0x00e6e6), description=faq.contents)


async def _search(sending: Sendable, faqs: List[FAQ], search: str) -> None:
    results: List[discord.Embed] = []
    for faq in faqs:
        if re.search(search, faq.search):
            results.append(_embed(faq))

    if len(results) > 0:
        LOG.info(f'event=faq search="{search}"')
        await sending.send(content=f"I found the following {len(results)} faq(s)", embeds=results)
    else:
        LOG.warning(f'event=faq.missing search="{search}"')
        await sending.send(content="Sorry, I did not find any faqs related to your search.\nPlease contribute to expand my faq list: <https://github.com/SquidDev-CC/FAQBot-CC>")


class FAQCog(commands.Cog):
    """
    Provides an %faq (or %f, %info, %i) command. This searches for a FAQ based
    on a user-provided regex.
    """

    def __init__(self, faqs: List[FAQ]):
        self.faqs = faqs

    @commands.command(name='faq', aliases=['f', 'info', 'i'])
    @with_async_timer(COMMAND_TIME.labels('faq', 'message'))
    async def faq(self, ctx: commands.Context, *, search):
        """Retrieves FAQs related to given keyword(s)."""
        await _search(SendableContext(ctx), self.faqs, search)

    @faq.error
    async def faq_error(self, ctx: commands.Context, error) -> None:
        """Error handler for the %faq command"""
        if isinstance(error, commands.MissingRequiredArgument):
            await ctx.send(content="Missing arguments! Please provide keywords to search for.")
        else:
            LOG.exception("Error processing faq command: %s", error)
            await ctx.send("An unexpected error occurred when processing the command.")


def _add_slash(slash: SlashCommand, faq: FAQ) -> None:
    @slash.subcommand(
        base="faq", name=faq.name,
        description=faq.title,
        guild_ids=guild_ids(),
    )
    @with_async_timer(COMMAND_TIME.labels('faq', 'slash'))
    async def _run(ctx: SlashContext) -> None:
        await ctx.send(embeds=[_embed(faq)])


def add_faq_slashcommands(slash: SlashCommand, faqs: List[FAQ]) -> None:
    """
    Register /faq subcommands for each FAQ.

    This is pretty ugly - it'd be nicer if we could do this with a Cog or something,
    but Discord's commands are still in beta, so nothing is really ready yet.
    """

    if len(faqs) <= 25:
        # Slash commands don't accept more than 25 options, so we only can
        # register it then. Thankfully we shouldn't hit that limit for a while.
        for faq in faqs:
            _add_slash(slash, faq)
    else:
        @slash.slash(
            name="faq", description="Retrieves FAQs related to given keyword(s).",
            options=[
                manage_commands.create_option(
                    name="search",
                    description="The FAQ to find.",
                    option_type=3,
                    required=True,
                ),
            ],
            guild_ids=guild_ids(),
        )
        @with_async_timer(COMMAND_TIME.labels('faq', 'slash'))
        async def _run(ctx: SlashContext, search: str) -> None:
            LOG.info(f'event=faq search="{faq.name}"')
            await _search(ctx, faqs, search)
