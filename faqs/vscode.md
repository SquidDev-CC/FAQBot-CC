---
title: Visual Studio Code setup for ComputerCraft
search: vscode
---
[Visual Studio Code](https://code.visualstudio.com) is the preferred code editor for writing ComputerCraft code. A number of extensions are available that improve CC programming:

- [Lua](https://marketplace.visualstudio.com/items?itemName=sumneko.lua) by sumneko adds Lua autocomplete, highlighting, and error checking.
- [ComputerCraft](https://marketplace.visualstudio.com/items?itemName=jackmacwindows.vscode-computercraft) by JackMacWindows adds autocomplete for CC functions.

These have been packaged into [one extension pack](https://marketplace.visualstudio.com/items?itemName=lemmmy.computercraft-extension-pack) by Lemmmy.

In addition, the [CraftOS-PC for VS Code](https://marketplace.visualstudio.com/items?itemName=jackmacwindows.craftos-pc) extension by JackMacWindows adds integration with CraftOS-PC, as well as a remote terminal and file viewer for standard CC.

By default, the Lua extension will highlight ComputerCraft functions with warnings since it doesn't know they exist. This can be fixed by adding them to the settings:

1. Press Ctrl+Shift+P (Command+Shift+P on Mac), type in "Preferences: Open Settings (JSON)", and press Enter.
2. Add the text [from this file](https://pastebin.com/fVTvu4Bw) to the top of the file, after the first `{`.
3. Close the file, and the warnings should go away.
