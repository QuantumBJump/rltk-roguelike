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
use voronoi::VoronoiCellBuilder;
mod prefab_builder;
use prefab_builder::*;

// Room-based meta builders
mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_starting_position;
use room_based_starting_position::RoomBasedStartingPosition;
mod room_based_stairs;
use room_based_stairs::RoomBasedStairs;
mod room_exploder;
use room_exploder::RoomExploder;

// Non-room-based meta builders
mod area_starting_points;
use area_starting_points::*;
mod voronoi_spawning;
use voronoi_spawning::VoronoiSpawning;
mod distant_exit;
use distant_exit::DistantExit;
mod cull_unreachable;
use cull_unreachable::CullUnreachable;

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

/// Chooses a random initial map generation algorithm.
/// # Returns
/// (builder: Box\<dyn InitialMapBuilder\>, has_rooms: bool)
fn random_initial_builder(rng: &mut rltk::RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);
    let result: (Box<dyn InitialMapBuilder>, bool);
    match builder {
        1 => result = (BspDungeonBuilder::new(), true),
        2 => result = (BspInteriorBuilder::new(), true),
        3 => result = (CellularAutomataBuilder::new(), false),
        4 => result = (DrunkardsWalkBuilder::open_area(), false),
        5 => result = (DrunkardsWalkBuilder::open_halls(), false),
        6 => result = (DrunkardsWalkBuilder::winding_passages(), false),
        7 => result = (DrunkardsWalkBuilder::fat_passages(), false),
        8 => result = (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => result = (MazeBuilder::new(), false),
        10 => result = (DLABuilder::walk_inwards(), false),
        11 => result = (DLABuilder::walk_outwards(), false),
        12 => result = (DLABuilder::central_attractor(), false),
        13 => result = (DLABuilder::insectoid(), false),
        14 => result = (VoronoiCellBuilder::pythagoras(), false),
        15 => result = (VoronoiCellBuilder::manhattan(), false),
        16 => result = (PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED), false),
        _ => result = (SimpleMapBuilder::new(), true)
    }
    result
}

/// Randomly generate a map
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomExploder::new());
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(RoomBasedStairs::new());
    builder
}
//     let mut builder = BuilderChain::new(new_depth);
//     let (random_starter, mut has_rooms) = random_initial_builder(rng);
//     builder.start_with(random_starter);

//     if rng.roll_dice(1, 3) == 1 {
//         // 1/3 chance of running through WFC algorithm
//         // Set has_rooms to false because if we run WFC on a room map
//         // The rooms will break
//         has_rooms = false;
//         builder.with(WaveformCollapseBuilder::new());
//     }

//     if has_rooms {
//         builder.with(RoomBasedSpawner::new());
//         builder.with(RoomBasedStairs::new());
//         builder.with(RoomBasedStartingPosition::new());
//     } else {
//         builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
//         builder.with(CullUnreachable::new());
//         builder.with(VoronoiSpawning::new());
//         builder.with(DistantExit::new());
//     }


//     if rng.roll_dice(1, 20) == 1 {
//         // 1/20 chance of an underground fort
//         builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
//     }

//     // Apply room vaults
//     builder.with(PrefabBuilder::vaults());

//     builder
// }
