---
title: Center-aligning text in ComputerCraft
search: centering text
---
There is no builtin function to write text center-aligned in ComputerCraft, but it is not hard to implement such a feature yourself.
```lua
local function centerWrite(text)
    local width, height = term.getSize() -- Get terminal size
    local x, y = term.getCursorPos() -- Get current cursor position
    local new_x = math.ceil((width / 2) - (text:len() / 2))
    term.setCursorPos(new_y, y)
    term.write(text)
end
```
After declaring this function, you can use it like this:
```lua
centerWrite("Hello world!")
```
This code starts by getting the current `term` object's size. Then it calculates the horizontal starting position for writing text by dividing the terminal width by 2, and subtracting half of the text length (the math.ceil it is to make sure the result is an integer). 
Finally, it moves the cursor to this x position and the current y position.

You may need to adapt this code to your situation, especially when you are working with non-default terminals, monitors, etc.
