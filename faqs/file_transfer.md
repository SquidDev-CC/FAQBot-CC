---
title: Transferring files to and from computers
search: file transfer open
---

ComputerCraft computers store their files inside a sandbox, meaning they can't open files on your real computer - only ones that are inside the computer's special folder. However, there are multiple ways to transfer files between your real computer and ComputerCraft.

**Drag & drop (1.16+)**
Since Minecraft 1.16, it's possible to simply drag and drop files from the file browser on top of the ComputerCraft console. This will upload the files you drag in to the computer, and will leave them in the top level directory with the same names as the real files. You cannot copy files back to your real computer this way, however.

**Save directory (singleplayer)**
If you are in a singleplayer world (or multiplayer where you own the server), you can easily open the original ComputerCraft files. The computer's directory is located at `.minecraft/saves/<world>/computercraft/computer/<id>`, where `<world>` is the save name of the world, and `<id>` is the ID of the computer, found with the `id` command.

**Pastebin**
Pastebin has been the de facto way to copy files between CC and real computers for a long time. To copy files into CC, go to https://pastebin.com, paste your file in, and then run `pastebin get <URL> <filename>` with the URL of Pastebin and the name you want to save it as. To go the other way, use `pastebin put <filename>` with the file you want to upload, then go to the URL it gives you and copy the data out. However, using it too much will block you for the rest of the day.

**Third-party services**
If you do not have access to the world, and you need to copy files both ways, you can use some third-party services. [cloud-catcher](https://cloud-catcher.squiddev.cc) allows you to open the computer's files and terminals in a web browser, and [CraftOS-PC Remote](https://remote.craftos-pc.cc) does the same for Visual Studio Code. Both of these have rich code editors too, making it easier to edit files. See the websites for more information on how to use them.
