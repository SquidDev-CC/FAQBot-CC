---
title: Too long without yielding
search: error too long without yielding
---
Computercraft has a builtin system which stops a single computer from running for too long at a time. After ~7 seconds of nonstop running, your computer will receive an error, "`Too long without yielding`."

If, a few seconds after your program received the initial error, your program still has not yielded (ie: the error was caught by `pcall` or `xpcall`), the computer itself will shut down.

**How do I fix this?**

First, you need to figure out where your program is looping. The error itself may not be too helpful, as the `Too long without yielding` error points to the last line that was run before timing out (which could be OS code, or some other library's code). In most cases, you're looking for a `while true do` loop.

Once you find where it's looping forever, you can simply add `sleep()` or `os.sleep()`. In most cases this will fix the problem. `sleep` has a call to a lua-builtin function, `coroutine.yield` which is what the error is referring to. `sleep` also has a bit of other event handling so that it can immediately resume.

Ex:

```lua
while true do
  -- Your code here
  ...
  sleep()
end
```

Some other yielding functions:
  - `os.pullEvent(_sFilter)`
    - Yields until an event occurs.
    - Calls `os.pullEventRaw` then checks if the event is a `terminate` event, invoking an error if it is.
  - `os.pullEventRaw(_sFilter)`
    - Exactly the same as `coroutine.yield(_sFilter)`.
  - `rednet.receive()`
    - Yields until a rednet message is received.
