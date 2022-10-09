---
title: How to enable HTTP on >1.12
search: atm7 http
---

**Singleplayer**
1. In CurseForge, go to "My Modpacks" and right-click the ATM-7 modpack.
2. Select "Open Folder" in the dropdown menu.
3. Open the folder titled `saves`.
4. Find your world folder, and open it.
5. Find a folder titled `serverconfig`, and open it.
6. Exit the game (or run `stop` on the server).
7. Search for a file named `computercraft-server.toml`, and open it in a text editor.
8. Search the file for a section starting with `[http]`.
9. Find `enabled` within this section, and set it to `true`.
10. Save the file and close the editor.

**Multiplayer**
If you're a player on the server, you will need to ask the server owner/operator to make these changes for you. 

(If you are a server owner)
1. Stop the server (run the command `stop`).
2. Go into the files of your server and go into ``world``
3. Open ``serverconfig``
4. Edit ``computercraft-server.toml``
5. Search for ``http``
6. Change ``enabled`` or ``enable`` to true