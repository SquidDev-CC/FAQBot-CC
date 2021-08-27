"""Runs code in an emulator and displays the result."""

import aiohttp
import asyncio
import io
import json
import logging
import re
from typing import List, Optional, Union
from attr import dataclass

import discord
from discord.message import Message
from discord_slash.context import ComponentContext
from discord_slash.model import ButtonStyle
import discord.ext.commands as commands
from discord_slash.cog_ext import cog_component
from discord_slash.utils.manage_components import create_button, create_actionrow

from ccfaq.commands import track_command
from ccfaq.config import eval_server


__all__ = ('EvalCog', )

LOG = logging.getLogger(__name__)

CODE_BLOCKS = re.compile(r'```(?:lua)?\n(.*?)```|(`+)(.*?)\2', flags=re.DOTALL | re.IGNORECASE)

DROP_COMMAND = re.compile(r'^%([a-z]+)')


async def message_reference(message: Message) -> Optional[Message]:
    if message.reference is None:
        return None

    if message.reference.cached_message is not None:
        return message.reference.cached_message

    if resolved := message.reference.resolved is not None:
        return resolved if isinstance(resolved, Message) else None

    msg_id = message.reference.message_id
    if msg_id is None:
        return None

    try:
        return await message.channel.fetch_message(msg_id)
    except:
        LOG.exception("Cannot resolve message")
        return None


@dataclass
class Result:
    message: str
    attachment: Optional[discord.File] = None


class EvalCog(commands.Cog):

    def __init__(self):
        self.session = aiohttp.ClientSession()

    def _get_code_blocks(self, message: Message) -> List[Union[str, discord.Attachment]]:
        contents = message.content

        code_blocks: List[Union[str, discord.Attachment]] = []
        for attachment in message.attachments:
            if attachment.content_type is not None and "text/plain" in attachment.content_type:
                # We could check file extension, but instead just rely on the fact that people won't do %eval on other
                # files.
                code_blocks.append(attachment)

        for code_block, _, inline_code in CODE_BLOCKS.findall(contents):
            code_blocks.append(code_block or inline_code)

        if code_blocks:
            return code_blocks

        contents = DROP_COMMAND.sub('', contents).strip()
        if contents:
            return [contents]

        return []

    async def _eval(self, message: Message) -> Result:
        """Execute some code."""

        code_blocks = self._get_code_blocks(message)

        if not code_blocks and message.reference:

            reply_to = await message_reference(message)
            if reply_to is None:
                return Result(":bangbang: No code found in message!")

            code_blocks = self._get_code_blocks(reply_to)

        if not code_blocks:
            return Result(":bangbang: No code found in message!")

        warnings = []
        if len(code_blocks) == 1:
            code = code_blocks[0]
        else:
            warnings.append(":warning: Multiple code blocks, choosing the first.")
            code = code_blocks[0]

        if isinstance(code, discord.Attachment):
            attachment: discord.Attachment = code

            try:
                code = (await attachment.read()).decode()
            except:
                LOG.exception("Error downloading attachment %s (%s)", attachment.filename, attachment.url)
                return Result(":bangbang: Error reading attachment.")

        if len(code) > 128 * 1024:
            # 128K is the same length as we use on nginx.
            return Result(":bangbang: Code block is too long to be run. Sorry!")

        LOG.info("Running %s", json.dumps(code))

        clean_exit = True
        try:
            response : aiohttp.ClientResponse
            async with self.session.post(eval_server(), data=code, timeout=aiohttp.ClientTimeout(total=20)) as response:
                if response.status == 200:
                    if response.headers.get("X-Clean-Exit") != "True":
                        clean_exit = False
                        warnings.append(":warning: Computer ran for too long.")

                    image = await response.read()
                else:
                    image = None
        except:
            LOG.exception("Error contacting eval.tweaked.cc")
            return Result(":bangbang: Unknown error when running code")

        LOG.info(f'event=eval has_image={image is not None} clean_exit={clean_exit}')

        if not image:
            return Result(":bangbang: No screenshot returned. Sorry!")
        else:
            return Result(
                message="\n".join(warnings),
                attachment=discord.File(io.BytesIO(image), 'image.png'),
            )

    @commands.command(name="eval", aliases=["exec", "code"])
    @track_command('eval', 'message')
    async def eval(self, ctx: commands.Context) -> None:
        result = await self._eval(ctx.message)
        if result.attachment is None:
            await ctx.reply(result.message, mention_author=False)
        else:
            await ctx.reply(
                result.message,
                mention_author=False,
                file=result.attachment,
                components=[create_actionrow(  # type: ignore
                    create_button(style=ButtonStyle.primary, label="Rerun", custom_id="on_rerun"),
                    create_button(emoji="🗑", style=ButtonStyle.danger, label="Delete", custom_id="on_delete"),
                )]
            )

    async def _resolve_origin(self, ctx: ComponentContext) -> Optional[Message]:
        original = None if ctx.origin_message is None else await message_reference(ctx.origin_message)
        if original is None:
            await ctx.reply("I can't remember anything about this message :/.", hidden=True)
            return None
        elif original.author != ctx.author:
            await ctx.send("Only the original commenter can do this. Sorry!", hidden=True)
            return None
        else:
            return original

    @cog_component()
    @track_command('eval', 'rerun')
    async def on_rerun(self, ctx: ComponentContext) -> None:
        if (message := await self._resolve_origin(ctx)) is None:
            return

        await ctx.defer(edit_origin=True)

        result = await self._eval(message)
        if result.attachment is None:
            await ctx.reply(result.message, hidden=True)
        else:
            # It'd be better to do edit_origin, but that doesn't accept attachments.
            resp = {
                'content': result.message,
                'attachments': [],
                'allowed_mentions': {},
            }
            await ctx._http.edit(resp, ctx._token, files=[result.attachment])
            ctx.deferred = False
            ctx.responded = True

    @cog_component()
    @track_command('eval', 'delete')
    async def on_delete(self, ctx: ComponentContext) -> None:
        if await self._resolve_origin(ctx):
            assert ctx.origin_message is not None
            await ctx.origin_message.delete()
            await ctx.defer(ignore=True)


    def cog_unload(self) -> None:
        asyncio.ensure_future(self.session.close())
