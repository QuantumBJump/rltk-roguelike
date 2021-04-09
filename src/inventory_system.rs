use specs::prelude::*;
use super::{
    WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog,
    ProvidesHealing, CombatStats, WantsToUseItem, WantsToDropItem,
    Consumable, InflictsDamage, Map, SufferDamage, AreaOfEffect,
    Stunned, Equippable, Equipped, WantsToRemoveItem,
    particle_system::ParticleBuilder, ProvidesFood, HungerClock,
    HungerState,
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            // Iterate through all entities which want to pick something up.
            positions.remove(pickup.item); // Remove the item from the gameworld
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry."); // Add to inventory

            if pickup.collected_by == *player_entity {
                // If picked up by player, log
                gamelog.entries.push(format!("You pick up the {}", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem{}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Stunned>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity, mut gamelog, map, entities, mut wants_use, names,
            consumables, healing, inflict_damage, mut combat_stats,
            mut suffer_damage, aoe, mut stunned, equippable, mut equipped,
            mut backpack, mut particle_builder, positions, provides_food,
            mut hungerclocks,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            let mut _used_item = true;

            // Targeting
            let mut targets: Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push( *player_entity ); }
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            // Single target in the tile
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.request(tile_idx.x, tile_idx.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('░'), 200.0);
                            }
                        }
                    }
                }
            }

            // If item is equippable, equip it, and unequip whatever else was in that slot.
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Remove any items the target has in the item's slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip the {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert item");
                    }

                    // Equip the item
                    equipped.insert(useitem.item, Equipped{ owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip the {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You use the {}, healing {} hp.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                            _used_item = true;

                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
                            }
                        }
                    }
                }
            }

            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    _used_item = true;
                    let target = targets[0];
                    let hc = hungerclocks.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!("You eat the {}", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    _used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, inflicting {} damage.", item_name.name, mob_name.name, damage.damage));

                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0
                                );
                            }
                        }

                        _used_item = true;
                    }
                }
            }

            let mut add_stun = Vec::new();
            {
                let causes_stun = stunned.get(useitem.item);
                match causes_stun {
                    None => {}
                    Some(stunned) => {
                        _used_item = false;
                        for mob in targets.iter() {
                            add_stun.push((*mob, stunned.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!("You use {} on {}, stunning them.", item_name.name, mob_name.name));

                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::MAGENTA), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                                }
                            }
                        }
                    }
                }
            }
            for mob in add_stun.iter() {
                stunned.insert(mob.0, Stunned{ turns: mob.1 }).expect("Unable to insert status.");
            }

            let consumable = consumables.get(useitem.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(useitem.item).expect("Delete failed!");
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem{}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut drop_intent, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &drop_intent).join() {
            let mut dropper_pos: Position = Position{x: 0, y: 0}; // Create outside scope
            {
                let dropped_pos = positions.get(entity).unwrap(); // Where is the holder?
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x: dropper_pos.y, y: dropper_pos.y }).expect("Unable to insert position!");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        drop_intent.clear();
    }
}

pub struct ItemRemoveSystem{}

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, player_entity, names, mut gamelog, mut wants_remove,
            mut equipped, mut backpack
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
            if entity == *player_entity {
                gamelog.entries.push(format!("You unequip the {}.", names.get(to_remove.item).unwrap().name));
            }
        }

        wants_remove.clear();
    }
}
