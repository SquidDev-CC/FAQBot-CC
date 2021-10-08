---
title: How to Ask a Question
search: ask question
---
Knowing how to effectively ask a question is essential in getting help from others. If you ask a vague question without many details, others have to slowly pry out what exactly you need. By stating everything up front, it's much easier to get answers because those who want to help don't have to ask a bunch of questions just to know what you want.

**1. Give a summary of your problem**  
State briefly what the issue you're running into is. Explain your goal and what problem is occurring. This should be no more than a few sentences; more detail will be added later.

**2. Show the code or program you're running**  
This is where you link to or embed the code you are using. If you're writing a program, paste in the code you wrote inside "\`\`\`lua" and "\`\`\`" brackets - this gives it some styling that makes it easier to read. (An example is listed below.) If you're using someone else's program, link the Pastebin or forum URL that you got it from.

**3. Explain what you want to have happen**  
Describe what you expect to happen with what you're trying. Explain what you're trying to accomplish too - in some cases, [what you want to accomplish may be easier to do another way rather than solving the current issue you're facing](https://en.wikipedia.org/wiki/XY_problem).

**4. Show what's actually happening**  
Here you should describe the issue fully, including any error messages you're getting. Show the specific difference between what you want to happen and what's actually happening, so others know why what you're getting is wrong.

**5. Add information about your environment**  
Tell what Minecraft and ComputerCraft versions you're running here. Also include any peripheral mods (Plethora, Advanced Peripherals) you have installed and their versions. Some issues, such as Pastebin not working, are tied to a specific version (1.7.10), and knowing this ahead of time will make it much easier to give an answer.

Here is an example of an effective question.

> [1] I'm having some trouble making the Parallel API work. I want to run two functions together, but only one is being called.  
> [2] Here is the code I'm using:  
> \`\`\`lua  
> ```lua
> local function eventLoop()  
>   while true do  
>     local event, key = os.pullEvent("key")  
>     if key == keys.a then print("Hello!") end  
>   end  
> end  
> local function printLoop()  
>   while true do  
>     term.setCursorPos(1, 1)  
>     term.clear()  
>     print("Time: " .. textutils.formatTime(os.time()))  
>     sleep(1)  
>   end  
> end  
> parallel.waitForAny(eventLoop(), printLoop())  
> ```
> \`\`\`  
> [3] I want to have the event loop and print loop functions running at the same time, so I can press A and have it say hello, while the screen also updates with the time every second.  
> [4] Instead, it doesn't show the time, and only prints hello when I press A.  
> [5] I'm running ComputerCraft 1.98.2 on Minecraft 1.16.5.  
