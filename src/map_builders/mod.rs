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
    // let mut rng = rltk::RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 8);
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     2 => Box::new(BspInteriorBuilder::new(new_depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(new_depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
    //     7 => Box::new(MazeBuilder::new(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth))
    // }

    Box::new(DLABuilder::new(new_depth))

}
