# FAQBot-CC
#
# Discord bot for private use only.
# Created by Wendelstein7, https://github.com/FAQBot-CC

print( "STARTING DISCORD BOT..." )

import discord
from discord.ext import commands

import os
import sys

import datetime
from datetime import datetime, date

import re

import faq_list

bot = commands.Bot( command_prefix='%' )
starttime = datetime.utcnow()
faqs = []


def msg_should_process( message ):  # do not process bot messages, own messages, non-default messages
    if (message.author.bot) or (message.author.id == bot.user.id) or (message.type is not discord.MessageType.default):  # or (message.guild is None):
        return False
    else:
        return True


@bot.event
async def on_ready():
    print( 'Bot: Logged in as {} (id: {})\n'.format( bot.user.name, bot.user.id ) )


@bot.event
async def on_message( message ):
    if msg_should_process( message ):
        await bot.process_commands( message )


@bot.event
async def on_command( ctx ):
    print( '[{} UTC] Fired {} by {}'.format( datetime.utcnow(), ctx.command, ctx.author ) )


@bot.command( name='faq', aliases=['f', 'info', 'i'] )
@commands.guild_only()
async def faq( ctx, *args ):
    """Retrieves answers related to given keyword(s)."""
    results = []
    for search in args:
        for f in faqs:
            if re.search( f[0], search ):
                results.append( discord.Embed( title=f[1], url="https://github.com/Wendelstein7/FAQBot-CC", colour=discord.Colour( 0x00e6e6 ), description=f[2] ) )
    if len( results ) > 0:
        await ctx.send( content="I found the following " + str( len( results ) ) + " faq(s)" )
        for result in results:
            await ctx.send( embed=result )
    else:
        await ctx.send( content="Sorry, I did not find any faqs related to your search.\nPlease contribute to expand my faq list: <https://github.com/Wendelstein7/FAQBot-CC>" )


@faq.error
async def faq_error( ctx, error ):
    if isinstance( error, commands.MissingRequiredArgument ):
        await ctx.send( content="Missing arguments! Please provide keywords to search for." )


@bot.command( name='about', aliases=[] )
async def about( ctx ):
    """Shows information about the bot aswell as the relevant version numbers, uptime and useful links."""
    embed = discord.Embed( title="ComputerCraft FAQ Bot", colour=discord.Colour( 0x00e6e6 ), url="https://github.com/Wendelstein7/FAQBot-CC", description="A Discord bot for answering frequently asked questions regarding CC. Please contribute and expand the list of answers on [GitHub](https://github.com/Wendelstein7/FAQBot-CC)!" )
    embed.set_thumbnail( url=bot.user.avatar_url )
    embed.add_field( name=":information_source: **Commands**", value="Please use the `%help` to list all possible commands.", inline=True )
    embed.add_field( name=":hash: **Developers**", value="**HydroNitrogen** as creator and other contributors mentioned on GitHub.", inline=True )
    embed.add_field( name=":new: **Version information**", value="Bot version: `{}`\nDiscord.py version: `{}`\nPython version: `{}`".format( date.fromtimestamp( os.path.getmtime( 'bot.py' ) ), discord.__version__, sys.version.split( ' ' )[0] ), inline=True )
    embed.add_field( name=":up: **Uptime information**", value="Bot started: `{}`\nBot uptime: `{}`".format( starttime.strftime( "%Y-%m-%d %H:%M:%S UTC" ), (datetime.utcnow().replace( microsecond=0 ) - starttime.replace( microsecond=0 )) ), inline=True )
    await ctx.send( embed=embed )


for faq in faq_list.faqs:
    try:
        print( 'Loading faqs\\' + faq[2] )
        file = open( 'faqs\\' + faq[2] )
        faqs.append( (faq[0], faq[1], file.read().strip()) )
    except IOError:
        print( 'Error when reading faq file...' )
    finally:
        file.close()
print( 'Successfully loaded ' + str( len( faqs ) ) + '!' )

with open( 'token', 'r' ) as file:
    content = file.read().strip()

bot.run( content )
