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

[mcm]: https://discord.computercraft.cc "The Minecraft Computer Mods Discord"
[eval.tweaked.cc]: https://github.com/cc-tweaked/eval.tweaked.cc
