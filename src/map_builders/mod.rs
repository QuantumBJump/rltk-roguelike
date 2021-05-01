use super::{
    Map, Rect, TileType, Position, World, spawner, SHOW_MAPGEN_VISUALISER,
};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
use drunkard::*;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::*;
mod common;
use common::*;
mod voronoi;
use voronoi::VoronoiBuilder;
mod prefab_builder;
use prefab_builder::*;

mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_starting_position;
use room_based_starting_position::RoomBasedStartingPosition;
mod room_based_stairs;
use room_based_stairs::RoomBasedStairs;

mod waveform_collapse;
use waveform_collapse::*;

/// BuilderMap stores shared state which can be accessed by various different builders.
pub struct BuilderMap {
    /// List of places to spawn things & what to spawn there: Vec<(tile_idx, entity_to_spawn)>
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    /// Which tile the player starts at
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
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

/// BuilderChain stores all the various chained map builders,
/// along with the shared state they need access to.
pub struct BuilderChain {
    /// The builder which creates the initial map state
    starter: Option<Box<dyn InitialMapBuilder>>,
    /// Further builders which modify the initial map
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            }
        }
    }

    /// Sets the initial map builder. Panics if starter is already set,
    /// as it only makes sense to have on initial map builder.
    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder!")
        };
    }

    /// Add a meta builder to the queue
    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    /// Build the map, by calling the initial builder, and then
    /// the metabuilders in order
    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system!"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

/// Builder which generates an initial map
pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

/// Builder which takes an existing map, and modifies it in some way
pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MapBuilder {
    // Generators
    fn build_map(&mut self);
    fn take_snapshot(&mut self);

    // Getters
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    // Defaults
    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(BspInteriorBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder
}

// pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
//     // randomly choose a type of map to build
//     let mut rng = rltk::RandomNumberGenerator::new();
//     let builder = rng.roll_dice(1, 17);
//     let mut result: Box<dyn MapBuilder>;
//     match builder {
//         1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
//         2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
//         3 => { result = Box::new(CellularAutomataBuilder::new(new_depth)); }
//         4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
//         5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
//         6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
//         7 => { result = Box::new(MazeBuilder::new(new_depth)); }
//         8 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
//         9 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
//         10 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
//         11 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
//         12 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
//         13 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
//         14 => { result = Box::new(VoronoiBuilder::pythagoras(new_depth)); }
//         15 => { result = Box::new(VoronoiBuilder::manhattan(new_depth)); }
//         16 => { result = Box::new(VoronoiBuilder::chebyshev(new_depth)); }
//         _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
//     }

//     if rng.roll_dice(1, 3) == 1 {
//         result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
//     }

//     if rng.roll_dice(1, 20) == 1 {
//         result = Box::new(PrefabBuilder::sectional(new_depth, prefab_builder::prefab_sections::UNDERGROUND_FORT, result));
//     }

//     result = Box::new(PrefabBuilder::vaults(new_depth, result));

//     result

// }
