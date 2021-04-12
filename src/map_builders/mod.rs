use super::{
    Map, Rect, TileType, Position, World, spawner, SHOW_MAPGEN_VISUALISER
};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

pub trait MapBuilder {
    // Generators
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn take_snapshot(&mut self);

    // Getters
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
}

pub fn random_builder(current_depth: i32) -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even remotely random
    Box::new(SimpleMapBuilder::new(current_depth))
}
