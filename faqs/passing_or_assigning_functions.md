**Passing or Assigning Functions**

A common problem for beginners when programming in lua is assigning a function to another variable, or calling a function which takes a function as a parameter.  Often, people will try doing the following:

```lua
local myFunction = turtle.forward()

local ok, error_message = pcall(turtle.forward())
```

Do you see what's wrong with the above?  In both assigning `myFunction` and using `turtle.forward` in `pcall`, this code calls the function first, then returns whatever the return value of the function itself is.

So, following this logic - `myFunction`'s value would be either `true` or `false`, since we would call `turtle.forward()`, then assign `myFunction` to the return value. That's probably not what we want, is it?  
Similarly, for `pcall(turtle.forward())`: we'll get an error stating, `attempt to call boolean value`. The reason for this is the same as above. We call `turtle.forward()`, then the return values are passed to `pcall`.
In reality, what we're really calling here is `pcall(true)`.

**How do I fix this?**

Fixing this is simple, remove the parenthesis!  The parenthesis can be seen as the cue to lua to actually call the function, without them we're just referring to the variable itself.

```lua
local myFunction = turtle.forward

local ok, error_message = pcall(turtle.forward)
```
