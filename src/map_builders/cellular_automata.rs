use super::{
    MapBuilder, Map, TileType, Position, spawner,
    SHOW_MAPGEN_VISUALISER,
};
use rltk::RandomNumberGenerator;
use rltk::DijkstraMap;
use specs::prelude::*;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        // TODO: implement
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALISER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map: Map::new(new_depth),
            starting_position: Position{ x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First we completely randomise the map, setting 55% of it to be floor.
        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                if roll > 55 { self.map.tiles[idx] = TileType::Floor }
                else { self.map.tiles[idx] = TileType::Wall }
            }
        }
        self.take_snapshot();

        // Now we iteratively apply cellular automata rules.
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height-1 {
                for x in 1..self.map.width-1 {
                    let idx = self.map.xy_idx(x, y);
                    // Count the tile's neighbours.
                    let mut neighbours = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall { neighbours += 1; } // west
                    if self.map.tiles[idx + 1] == TileType::Wall { neighbours += 1; } // east
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall { neighbours += 1; } // north
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall { neighbours += 1; } // south
                    if self.map.tiles[idx - self.map.width as usize - 1] == TileType::Wall { neighbours += 1; } // northwest
                    if self.map.tiles[idx - self.map.width as usize + 1] == TileType::Wall { neighbours += 1; } // northeast
                    if self.map.tiles[idx + self.map.width as usize - 1] == TileType::Wall { neighbours += 1; } // southwest
                    if self.map.tiles[idx + self.map.width as usize + 1] == TileType::Wall { neighbours += 1; } // southeast

                    if neighbours > 4 || neighbours == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }

                }
            }
            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point using a simple algorithm;
        // Start in the middle & walk left until you find an open tile.
        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // Find all tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = DijkstraMap::new(self.map.width, self.map.height, &map_starts, &self.map, 200.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start == std::f32::MAX {
                    // We can't get to this tile - so we'll make it a wall
                    *tile = TileType::Wall;
                } else {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.take_snapshot();
    }
}