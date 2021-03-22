"""
Discord bot for private use only.
Created by Wendelstein7
"""

import logging

import discord
from discord.ext.commands import Bot
from discord_slash import SlashCommand

import ccfaq.faq_list
import ccfaq.config
from ccfaq.commands.about import AboutCog
from ccfaq.commands.docs import DocsCog
from ccfaq.commands.faq import FAQCog, add_faq_slashcommands

LOG = logging.getLogger(__name__)

bot = Bot(command_prefix='%')


@bot.event
async def on_ready() -> None:
    LOG.info('Bot: Logged in as %s (id: %s)', bot.user.name, bot.user.id)


@bot.event
async def on_message(message: discord.Message) -> None:
    if message.author.bot or message.author.id == bot.user.id or message.type is not discord.MessageType.default:
        # do not process bot messages, own messages, non-default messages
        return

    await bot.process_commands(message)


@bot.event
async def on_command(ctx):
    LOG.info('Fired %s by %s', ctx.command, ctx.author)


def run() -> None:
    faqs = ccfaq.faq_list.load()
    LOG.info('Successfully loaded %d FAQs!', len(faqs))

    slash = SlashCommand(bot, sync_commands=True)

    bot.add_cog(AboutCog(bot, faqs))
    bot.add_cog(DocsCog())
    bot.add_cog(FAQCog(faqs))

    add_faq_slashcommands(slash, faqs)

    LOG.info("Starting discord Bot")
    bot.run(ccfaq.config.token())
