use specs::prelude::*;
use super::{
    EntityMoved, Position, EntryTrigger, Hidden, Map, Name, gamelog::GameLog,
};

pub struct TriggerSystem{}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map, mut entity_moved, positions, entry_triggers, mut hidden, names,
            entities, mut gamelog,
        ) = data;

        // For each entity which moved, look at its final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &positions).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            // Iterate through all other entities on that tile to look for triggered entities.
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id { // Don't bother to check whether you are a trap.
                    let maybe_trigger = entry_triggers.get(*entity_id);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            // We triggered it!
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                gamelog.entries.push(format!("{} triggers!", &name.name));
                            }

                            hidden.remove(*entity_id); // The trap is no longer hidden.
                        }
                    }    
                }
            }
        }

        // Remove all entity movement markers.
        entity_moved.clear();
    }
}