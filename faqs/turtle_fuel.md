---
title: Turtle fuel
search: turtle refueling getFuelLevel
---
**The principles of fueling ComputerCraft Turtles**

By default, ComputerCraft Turtles need fuel to move, *but not to rotate*.
Without fuel, actions like `turtle.forward()` will not execute and return `false`.
Moving will consume 1 fuel unit per block moved.

**Turtle refueling:**
You can refuel a turtle by inserting a Minecraft fuel item like lava or coal into its inventory.
By executing `turtle.refuel()`, the Turtle will attempt to refuel by consuming items from the currently selected slot.
*See [turtle.refuel on wiki.computercraft.cc](https://wiki.computercraft.cc/Turtle.refuel) for more information.*

**Turtle fuel checking:**
You can use `turtle.getFuelLevel()` to get the amount of fuel units (block moves) the turtle currently holds.
*See [turtle.getFuelLevel on wiki.computercraft.cc](https://wiki.computercraft.cc/Turtle.getFuelLevel) for more information.*


*Fuel requirement can be turned off in `config/computercraft.cfg` by setting `need_fuel` to `false`.*
