---
title: How to enable HTTP on ATM7
search: atm7
---

**Singleplayer**
1. Press the windows key and R at the same time
2. Type in ``%appdata%\.minecraft``
3. Go into ``saves`` then the name of your save/world
4. Then go into serverconfig and search for ``computercraft-server.toml``
5. Open the file in notepad or any other text editor and search for ``[http]``
6. There you should find ``enabled``, that should be set to false
7. Set the option to true and save the file
8. Your done

**Multiplayer**
(If your a player)
1. Ask the server owner or someone who has access to the files to activate http
2. Hope they do it and not deny it cause of "security"

(If your a server owner)
1. Go into the files of your server and go into ``config``
2. Open ``computercraft.cfg``
3. Search for ``http``
4. Change ``enabled`` or ``enable`` to true