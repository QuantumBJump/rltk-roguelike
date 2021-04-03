use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers) = data;

        map.populate_blocked(); // all walls are blocked
        for (position, _blocks) in (&position, &blockers).join() {
            // All entities on the map which block movement
            let idx = map.xy_idx(position.x, position.y);
            map.blocked[idx] = true;
        }
    }
}