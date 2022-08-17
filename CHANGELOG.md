# v0.6.0
## Features
* Added an XP system and leveling up
  * Added xp bar
  * Added particle effects on level up
* Added help menu
## Bugfixes
* Animals are no longer immune to being stunned
* The New Game option now works even in the middle of a game session

# v0.5.0
## Features
* Added forest level outside town
  * Added deer, wolves, and foxes
  * Added bandits
* Combat system
  * Enemies can now drop items when they die
* Improved UI
* Added options system using options.json
* Added WASD keybindings
* Refactored player input code
## Bugfixes
* Bystanders no longer phase through the player
* Monsters no longer constantly report the state of their memory

# v0.4.0 (2021/05/25)
## Features
* Added raw file decoder to allow for easier definition of spawnable entities
* Added a fishing town for your adventure to start in, containing:
    * A pub
    * A temple
    * A blacksmith, clothier, and alchemist
    * Your mum's house
    * Several peasant houses
    * An abandoned house filled with rats!
* Townsfolk will sometimes speak their mind when near you
* Reworked melee combat to use a d20-like system
    * Implemented armour and weapons
    * Implemented spawning creatures with inventories
    * Implemented natural armour and weaponry (Creatures with multiple natural weapons will choose one)
## Bugfixes
* You no longer accidentally murder your mother when you bump into her
* Monsters no longer pursue the player forever
* Monsters can now see through open doors
* The abandoned house in the starting village is no longer filled with an inordinate number of rats

# v0.3.0 (2021/05/20)
## Features
* Factored map building out into a module to improve extensibility
* Added several new types of map generation
    * Sewer level using Binary Space Partition
    * Castle level using Binary Space Partition
    * Cellular Automata cave
    * Several Drunkard's Walk maps
        * Main central cavern
        * Large, open hallways
        * Tighter, more winding caves
    * Labyrinth
    * Diffusion Limited Aggregation
    * Hive maps
    * Added prefabs
* Refactored map gen code to be more composable
* Added doors
* Added moving camera

# v0.2.0
## Features
* Improved wall tiles with bitsets
* Added bloodstains
* Added particle effects in various situations
* Added hunger and rations
* Added a Magic Map scroll
* Improved main menu
* Added bear traps
