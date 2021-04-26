use super::{
    MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALISER,
    generate_voronoi_spawn_regions, get_most_distant_area,
};
use rltk::RandomNumberGenerator;
use std::collections::HashMap;
use specs::prelude::*;

mod image_loader;
use image_loader::*;

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for WaveformCollapseBuilder {
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

impl WaveformCollapseBuilder {
    pub fn new(new_depth: i32) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder {
            map: Map::new(new_depth),
            starting_position: Position{x: 0, y: 0},
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.map = load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../resources/wfc-demo1.xp").unwrap());
        self.take_snapshot();

        // Find a starting point; start at the middle & walk left until we find an open space
        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = get_most_distant_area(&mut self.map, start_idx, true);
        self.take_snapshot();

        // Place stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Build a noise map
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
