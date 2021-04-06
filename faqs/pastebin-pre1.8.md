---
title: Pastebin on old versions of ComputerCraft
search: old pastebin pre1.8
---
Pastebin changed the way files are downloaded a few years ago. This means that old versions of the `pastebin` program (1.75 and lower for Minecraft 1.7.0 or lower) can't download files anymore without a patch.

Here are two methods you can use to fix the `pastebin` program and make it work on old versions of ComputerCraft.

**Using a resource pack (MC 1.6.1 - 1.7.10)**
This method fixes the program for all computers (in all worlds on singleplayer), but it requires the server owner to install a resource pack if used for a server. Simply download [this resource pack](https://raw.githubusercontent.com/SquidDev-CC/FAQBot-CC/master/etc/cc-pastebin-fix.zip) and install it into your `resourcepacks` folder, or into the server's installation directory. Then restart Minecraft and apply the resource pack. After that, the default `pastebin` program should function normally.

**Manually patching the program**
This method only fixes the program on one computer, and must be run for each computer that needs to be patched, but it doesn't require modifying Minecraft in any way, and works fine on servers. First, copy the default `pastebin` program to the root directory of your computer, and then edit it with these commands:
```
copy /rom/programs/http/pastebin /pastebin
edit /pastebin
```
Then go to line 24, and replace this:
```
"http://pastebin.com/raw.php?i="..textutils.urlEncode( paste )
```
with this:
```
"https://pastebin.com/raw/"..textutils.urlEncode( paste )
```
After that, you can run the pastebin program at `/pastebin`.

[This information was taken from a post on the ComputerCraft forums.](http://www.computercraft.info/forums2/index.php?/topic/26882-resource-pack-pastebin-fix-for-pre-mc18x-users/)
