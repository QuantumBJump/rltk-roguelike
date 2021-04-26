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

mod waveform_collapse;
use waveform_collapse::*;

pub trait MapBuilder {
    // Generators
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn take_snapshot(&mut self);

    // Getters
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // randomly choose a type of map to build
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 18);
    let mut result: Box<dyn MapBuilder>;
    match builder {
        1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(CellularAutomataBuilder::new(new_depth)); }
        4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(MazeBuilder::new(new_depth)); }
        8 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
        9 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
        10 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
        11 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
        12 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
        13 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        14 => { result = Box::new(VoronoiBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(VoronoiBuilder::manhattan(new_depth)); }
        16 => { result = Box::new(VoronoiBuilder::chebyshev(new_depth)); }
        17 => { result = Box::new(WaveformCollapseBuilder::test_map(new_depth)); }
        _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    result
}
