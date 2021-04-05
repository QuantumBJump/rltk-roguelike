use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog };

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
