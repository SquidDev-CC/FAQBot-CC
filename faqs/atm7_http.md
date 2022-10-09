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
6. Search for a file named `computercraft-server.toml`, and open it in a text editor.
7. Search the file for a section starting with `[http]`.
8. Find `enabled` within this section, and set it to `true`.
9. Save the file and close the editor, then save and quit to title (or `stop` the server), then relaunch the world.

**Multiplayer**
(If you're a player)
1. Ask the server owner or someone who has access to the files to activate http
2. Hope they do it and not deny it cause of "security"

(If you're a server owner)
1. Go into the files of your server and go into ``world``
2. Open ``serverconfig``
3. Edit ``computercraft-server.toml``
4. Search for ``http``
5. Change ``enabled`` or ``enable`` to true