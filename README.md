# FAQBot-CC
A Discord bot for getting help with ComputerCraft.

This bot is used in the [Minecraft Computer Mods][mcm] Discord Server.

## Features
 - Answers some frequently asked questions (see `./faqs` for a list).
 - Link to the documentation and source code of built-in ComputerCraft definitions.
 - Run snippets of code using [eval.tweaked.cc]

## Using
Due to requiring the message content intent, this bot does not have a public
instance, and must be self-hosted.

 - Run `dotnet publish -c Release --self-contained true -r linux-x64` to build
   the project. The resulting executable is written to
   `./bin/Release/net6.0/linux-x64/publish/FAQBot-CC`.
 - Create `config.json` file containing `{"token": "<your discord token>"}`.
 - Run the bot with `./FAQBot-CC`.

## Terms of Service
The bot has no additional terms of service on top of Discord's own.

## Privacy Policy
The bot logs some information in the event of an error. This includes the
username of the user who interacted with the bot, but does *not* include message
content. This information is used for debugging purposes, and is not shared with
any third parties.

You can [contact the developer via email or IRC](https://squiddev.cc/) if you
have any concerns.

[mcm]: https://discord.computercraft.cc "The Minecraft Computer Mods Discord"
[eval.tweaked.cc]: https://github.com/cc-tweaked/eval.tweaked.cc
