use super::{Map, Rect, TileType, Position, World, spawner};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(current_depth: i32) -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even remotely random
    Box::new(SimpleMapBuilder::new(current_depth))
}
