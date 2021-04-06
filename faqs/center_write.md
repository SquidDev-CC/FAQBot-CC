---
title: Center-aligning text in ComputerCraft
search: centering text
---
There is no builtin function to write text center-aligned in ComputerCraft, but it is not hard to implement such a feature yourself.
```lua
local function centerWrite(text)
    local width, height = term.getSize() -- Get terminal size
    local x, y = term.getCursorPos() -- Get current cursor position
    term.setCursorPos(math.ceil((width / 2) - (text:len() / 2)), y)
    term.write(text)
end
```
After declaring this function, you can use it like this:
```lua
centerWrite("Hello world!")
```
This code gets the current `term` objects size. Now, to get a starting point for writing text, it divides the `term` width by 2. It then subtracts half of the text length, and what results is the x value to start writing the text from to appear 'centered'.
You may need to adapt this code to your situation, especially when you are working with non-default terminals, monitors, etc.
