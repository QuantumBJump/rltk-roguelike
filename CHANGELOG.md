# v0.4.0
## Features
* Added raw file decoder to allow for easier definition of spawnable entities
* Added a fishing town for your adventure to start in, containing:
    * A pub
    * A temple
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