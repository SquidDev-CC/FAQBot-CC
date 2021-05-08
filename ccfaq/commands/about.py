"""Provides commands about the bot itself."""

from typing import List
from datetime import datetime, date
import logging
import os
import sys

import discord
import discord.ext.commands as commands

from ccfaq.faq_list import FAQ


LOG = logging.getLogger(__name__)


class AboutCog(commands.Cog):
    """
    Provides an %about command to provide some information about the bot.
    """

    def __init__(self, bot: commands.Bot, faqs: List[FAQ]):
        self.bot = bot
        self.faqs: int = len(faqs)
        self.start_time = datetime.utcnow()

    @commands.command(name='about')
    async def faq(self, ctx: commands.Context):
        """Shows information about the bot as well as the relevant version numbers, uptime and useful links."""
        embed = discord.Embed(
            title="ComputerCraft FAQ Bot", colour=discord.Colour(0x00e6e6), url="https://github.com/SquidDev-CC/FAQBot-CC",
            description="A Discord bot for answering frequently asked questions regarding CC. Please contribute and expand the list of answers on [GitHub](https://github.com/SquidDev-CC/FAQBot-CC)!"
        )
        embed.set_thumbnail(url=str(self.bot.user.avatar_url))
        embed.add_field(
            name=":information_source: **Commands**",
            value="Please use the `%help` to list all possible commands.\nUse `%f <search>` to find faqs related to your search.",
            inline=True
        )
        embed.add_field(
            name=":hash: **Developers**",
            value="**HydroNitrogen** as creator and other contributors mentioned on GitHub.", inline=True
        )
        embed.add_field(
            name=":asterisk: **FAQs**",
            value=f"Currently there are {self.faqs} FAQs loaded into memory.",
            inline=True
        )
        embed.add_field(
            name=":up: **Uptime information**",
            value="Bot started: `{}`\nBot uptime: `{}`".format(
                self.start_time.strftime("%Y-%m-%d %H:%M:%S UTC"),
                (datetime.utcnow().replace(microsecond=0) -
                 self.start_time.replace(microsecond=0))
            ),
            inline=True
        )
        await ctx.send(embed=embed)
