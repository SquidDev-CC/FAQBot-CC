---
title: How to enable HTTP on >1.12
search: atm7 http
---

**Singleplayer**
1. Launch Minecraft.
2. Navigate to the Singleplayer menu and highlight your world.
3. Press the "Edit" button on the bottom, then "Open World Folder"
4. Find a folder titled `serverconfig`, and open it.
5. Exit the game.
6. Search for a file named `computercraft-server.toml`, and open it in a text editor.
7. Search the file for a section starting with `[http]`.
8. Find `enabled` within this section, and set it to `true`.
9. If you wish to also have access to websockets, look for `websocket_enabled` and ensure that it is `true` as well.
10. Save the file and close the editor.

**Multiplayer**
If you're a player on the server, you will need to ask the server owner/operator to make these changes for you.

1. Stop the server (run the command `stop`).
2. Go into the files of your server, then go into ``world``
3. Open ``serverconfig``
4. Edit ``computercraft-server.toml``
5. Search the file for a section starting with `[http]`.
6. Find `enabled` within this section, and set it to `true`.
7. If you wish to also have access to websockets, look for `websocket_enabled` and ensure that it is `true` as well.
8. Save the file and close the editor.
