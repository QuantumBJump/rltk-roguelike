use specs::prelude::*;
use super::{
    EntityMoved, Position, EntryTrigger, Hidden, Map, Name, gamelog::GameLog,
    InflictsDamage, particle_system::ParticleBuilder, SufferDamage,
    SingleActivation, Renderable,
};

pub struct TriggerSystem{}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, SingleActivation>,
        WriteStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map, mut entity_moved, positions, mut entry_triggers, mut hidden,
            names, entities, mut gamelog, inflicts_damage, mut particle_builder,
            mut inflict_damage, mut single_activation, mut renderable,
        ) = data;

        let mut deactivate_entities: Vec<Entity> = Vec::new();
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

                            // If the trap is damaging, inflict damage
                            let damages = inflicts_damage.get(*entity_id);
                            if let Some(damages) = damages {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                                SufferDamage::new_damage(&mut inflict_damage, entity, damages.damage, false);
                            }

                            // If it is a single activation, mark it for deactivation
                            let sa = single_activation.get(*entity_id);
                            if let Some(_sa) = sa {
                                deactivate_entities.push(*entity_id);
                            }
                        }
                    }    
                }
            }
        }

        // Deactivate any single use traps
        for trap in deactivate_entities.iter() {
            single_activation.remove(*trap);
            entry_triggers.remove(*trap);
            let entity_renderable = renderable.get_mut(*trap);
            if let Some(entity_renderable) = entity_renderable {
                entity_renderable.fg = rltk::RGB::named(rltk::GREY);
            }
        }

        // Remove all entity movement markers.
        entity_moved.clear();
    }
}