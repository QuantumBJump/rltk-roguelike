use super::{
    MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALISER,
    get_most_distant_area, generate_voronoi_spawn_regions
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

pub struct _Builder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for _Builder {
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
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
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

impl _Builder {
    pub fn new(new_depth: i32) -> _Builder {
        _Builder {
            map: Map::new(new_depth),
            starting_position: Position{ x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new()
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Find a starting point; start at the middle & walk left until we find an opening.
        self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
        self.take_snapshot();

        // Find all the tiles we can reach from the starting point.
        let exit_tile = get_most_distant_area(&mut self.map, start_idx, true);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();
        
        // Now build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
