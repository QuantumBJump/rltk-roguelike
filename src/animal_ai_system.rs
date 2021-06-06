use specs::prelude::*;
use super::{
    Viewshed, Herbivore, Carnivore, Item, Map, Position, WantsToMelee, RunState,
    Stunned, particle_system::ParticleBuilder, EntityMoved
};
use rltk::{Point};

pub struct AnimalAI {}

impl<'a> System<'a> for AnimalAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Herbivore>,
        ReadStorage<'a, Carnivore>,
        ReadStorage<'a, Item>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Stunned>,
        WriteExpect<'a, ParticleBuilder>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map, player_entity, runstate, entities, mut viewshed,
            herbivore, carnivore, item, mut wants_to_melee, mut entity_moved,
            mut position, mut stunned, mut particle_builder, mut rng
        ) = data;

        if *runstate != RunState::MonsterTurn { return; }

        // Herbivores run away a lot
        for (entity, mut viewshed, _herbivore, mut pos) in (&entities, &mut viewshed, &herbivore, &mut position).join() {
            let mut can_act = true;

            let is_stunned = stunned.get_mut(entity);
            if let Some(i_am_stunned) = is_stunned {
                i_am_stunned.turns -= 1;
                if i_am_stunned.turns < 1 {
                    stunned.remove(entity);
                }
                can_act = false;

                particle_builder.request(
                    pos.x,
                    pos.y,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0
                );
            }

            if can_act {
                let mut run_away_from: Vec<usize> = Vec::new();
                for other_tile in viewshed.visible_tiles.iter() {
                    let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                    for other_entity in map.tile_content[view_idx].iter() {
                        // They don't run away from items or other herbivores
                        if item.get(*other_entity).is_none() && herbivore.get(*other_entity).is_none() {
                            // The herbivore might not run away anyway - they might not notice
                            let run_roll = rng.roll_dice(1, 6);
                            if run_roll > 1 { // 5/6 chance of running
                                run_away_from.push(view_idx);
                            }
                        }
                    }
                }

                if !run_away_from.is_empty() {
                    let my_idx = map.xy_idx(pos.x, pos.y);
                    map.populate_blocked();
                    let flee_map = rltk::DijkstraMap::new(map.width as usize, map.height as usize, &run_away_from, &*map, 100.0);
                    let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
                    if let Some(flee_target) = flee_target {
                        if !map.blocked[flee_target] {
                            map.blocked[my_idx] = false; // We no longer block the square we're leaving
                            map.blocked[flee_target] = true; // We block the square we're entering
                            viewshed.dirty = true; // Recalculate FoV
                            // Update position
                            pos.x = flee_target as i32 % map.width;
                            pos.y = flee_target as i32 / map.width;
                            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                        }
                    }
                }
            }
        }

        // Carnivores just want to eat everything
        for (entity, mut viewshed, _carnivore, mut pos) in (&entities, &mut viewshed, &carnivore, &mut position).join() {
            let mut can_act = true;

            let is_stunned = stunned.get_mut(entity);
            if let Some(i_am_stunned) = is_stunned {
                i_am_stunned.turns -= 1;
                if i_am_stunned.turns < 1 {
                    stunned.remove(entity);
                }
                can_act = false;

                particle_builder.request(
                    pos.x,
                    pos.y,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0
                );
            }

            if can_act {
                let mut run_towards: Vec<usize> = Vec::new();
                let mut attacked = false;
                for other_tile in viewshed.visible_tiles.iter() {
                    let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                    for other_entity in map.tile_content[view_idx].iter() {
                        if herbivore.get(*other_entity).is_some() || *other_entity == *player_entity {
                            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                                Point::new(pos.x, pos.y),
                                *other_tile
                            );
                            if distance < 1.5 {
                                wants_to_melee.insert(entity, WantsToMelee{ target: *other_entity }).expect("Unable to insert intent");
                                attacked = true;
                            } else {
                                run_towards.push(view_idx);
                            }
                        }
                    }
                }

                if !run_towards.is_empty() && !attacked {
                    let my_idx = map.xy_idx(pos.x, pos.y);
                    map.populate_blocked();
                    let chase_map = rltk::DijkstraMap::new(map.width as usize, map.height as usize, &run_towards, &*map, 100.0);
                    let chase_target = rltk::DijkstraMap::find_lowest_exit(&chase_map, my_idx, &*map);
                    if let Some(chase_target) = chase_target {
                        if !map.blocked[chase_target] {
                            map.blocked[my_idx] = false;
                            map.blocked[chase_target] = true;
                            viewshed.dirty = true;
                            pos.x = chase_target as i32 % map.width;
                            pos.y = chase_target as i32 / map.width;
                            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                        }
                    }
                }
            }
        }
    }
}
