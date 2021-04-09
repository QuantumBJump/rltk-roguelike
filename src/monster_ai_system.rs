use specs::prelude::*;
use super::{
    Viewshed, Monster, RunState, WantsToMelee, Map, Position, Stunned,
    particle_system::ParticleBuilder,
};
use rltk::{Point};

pub struct MonsterAI {}

impl <'a> System<'a> for MonsterAI {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, Stunned>,
                        WriteExpect<'a, ParticleBuilder>,
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map, player_pos, player_entity, runstate, entities,
            mut viewshed, monster, mut position, mut wants_to_melee,
            mut stunned, mut particle_builder,
        ) = data;

        if *runstate != RunState::MonsterTurn { return; } // Only move on monster's turn.

        for (entity, mut viewshed, _monster, mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
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
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity}).expect("Unable to insert attack.");
                }
                else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &mut *map,
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }

            }
        }
    }
}