use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>,);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked(); // all walls are blocked
        map.clear_content_index(); // Clear the index of entities

        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If the entity blocks, update the blocking list
            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push the entity to the appropriate index slot. It's a Copy type,
            // So we don't need to clone it (we want to avoid moving it out of the
            // ECS!)
            map.tile_content[idx].push(entity);

        }
        for (position, _blocks) in (&position, &blockers).join() {
            // All entities on the map which block movement
            let idx = map.xy_idx(position.x, position.y);
            map.blocked[idx] = true;
        }
    }
}