---
title: Differences between CC, CC:Tweaked, OC, and OC2
search: differences different
---
Over the years, multiple different computer mods have been created. Each of these mods have their own unique features, and cater to different play styles. Below is a brief overview of the differences between the most popular computer mods.

## Brief Descriptions

**CC (ComputerCraft)**

ComputerCraft is the original version of ComputerCraft, developed by Dan200. It's available for Minecraft 1.6.4 up to 1.12.2. This version is missing many of the modern features that CC:Tweaked contains. The computers are programmed using Lua 5.1.

**CC:Tweaked**

CC:Tweaked is an updated version of ComputerCraft, maintained by SquidDev and many members of the community. It adds many new features on top of the original ComputerCraft, including websocket support, new peripherals, and DFPWM audio support. CC:Tweaked operates on the value of backwards compatibility, attempting to ensure that as many existing programs as possible continue to work without modification. Newer versions of the mod also include support for Lua 5.2.

**OC (OpenComputers)**

OpenComputers is a separate mod from ComputerCraft, developed by Asie. It focuses on a more 'realistic' approach to computers within Minecraft, featuring more complex (but limited) hardware components, and its own set of peripherals. OpenComputers is available for Minecraft 1.6.4 up to 1.12.2, and is also programmed using Lua.

**OC2 (OpenComputers 2)**

OC2 is the successor to OpenComputers, though make no mistake: it's not just a continuation like CC:Tweaked is. OC2 introduces the RISC-V architecture, which is a significant departure from the original OpenComputers mod. This mod is still in development, and aims to bring modern features and improvements to the OpenComputers experience. OC2 is available for Minecraft 1.18.1 and 1.18.2. OC2's default operating system is Linux, and Lua is included to enable easy scripting.

## Main Differences

If you're looking for a bullet-point list of differences between the mods, here you go:

- **Programming Language**: All four mods use Lua, but OC2 uses RISC-V, meaning it can use anything else you desire as well.
- **Operating System**
  - **CC/CC:Tweaked**:
    - CraftOS: Simplifies scripting for users as it provides a large set of APIs available in the global environment.
  - **OpenComputers**:
    - OpenOS: A more modular operating system that requires the user to explicitly import non-standard libraries.
  - **OC2**:
    - Base operating system is Linux, but you can theoretically run anything on it.
- **Resource Limits**
  - **CC/CC:Tweaked**:
    - No limits on CPU or RAM usage.
    - Configurable storage limit applied globally to all computers (and floppy disks).
  - **OpenComputers/OC2**:
    - Resource limits are determined by the hardware components used in the computer.
    - Tiered items are used to increase the available resources individually.
- **Complexity**:
  - **CC/CC:Tweaked**: Generally simpler to use and understand, requiring minimal crafting and setup to get started.
  - **OpenComputers/OC2**: Have a steeper learning curve due to their more complex hardware and software systems.
- **Realism**: 
  - **CC/CC:Tweaked**: While still offering a small degree of realism, are more focused on accessibility and ease of use.
  - **OpenComputers/OC2**: Aim for a much more realistic approach to computing within Minecraft, featuring more complex hardware components and a greater emphasis on modularity and customization.
- **World Interaction**:
  - **CC/CC:Tweaked**: Allow for basic interaction with the Minecraft world, primarily through the use of peripherals, turtles, and redstone.
    - **Peripherals**: Blocks which are placed in the world and connect to a computer, which can interact with the world in various ways (e.g., setting redstone states, playing audio).
    - **Turtles**: Mobile computers that can be programmed to perform tasks in the world, such as mining or building. Turtles can move anywhere (so long as they are not obstructed by another block).
  - **OpenComputers/OC2**: Require more setup and configuration to interact with the world, through the use of peripherals, robots, drones, and redstone components.
    - **Peripherals**: Similar to CC, OpenComputers provides many of their own peripherals that can interact with the world.
    - **Robots**: Mobile computers that can be programmed to perform tasks in the world, similar to turtles, but with more customization options. Robots have a [defined set of rules](https://ocdoc.cil.li/api:robot#movement) restricting their movement based on their surroundings.
  - **OpenComputers**:
    - **Drones**: Entities which can move through the air extremely quickly and perform small tasks.