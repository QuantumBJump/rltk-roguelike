use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{
    Player, Renderable, Name, Position, Viewshed, Rect,
    SerializeMe, random_table::RandomTable, HungerClock, HungerState, Map,
    TileType, raws::*, Attributes, Attribute, Skills, Skill, Pools, Pool,
};
use crate::{ attr_bonus, player_hp_at_level, mana_at_level};
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns their entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let mut skills = Skills{ skills: HashMap::new() };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);

    let player = ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name{ name: "Player".to_string() })
        .with(HungerClock{
            state: HungerState::WellFed,
            duration: 500, // TODO: change back to 20
        })
        .with(Attributes{
            might: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11)},
            fitness: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11)},
            quickness: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11)},
            intelligence: Attribute{ base: 11, modifiers: 0, bonus: attr_bonus(11)}
        })
        .with(skills)
        .with(Pools{
            hit_points: Pool{
                current: player_hp_at_level(51, 1), //TODO: change back to 11
                max: player_hp_at_level(11, 1)
            },
            mana: Pool{
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1)
            },
            xp: 0,
            level: 1,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Starting equipment
    // TODO: remove Tower Shield/Longsword, return Rusty Longsword
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Longsword", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Tower Shield", SpawnType::Equipped{ by: player });
    // spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Rusty Longsword", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Dried Sausage", SpawnType::Carried{by: player});
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Beer", SpawnType::Carried{by: player});
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Stained Tunic", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Torn Trousers", SpawnType::Equipped{ by: player });
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Old Boots", SpawnType::Equipped{ by: player });

    player
}

const MAX_SPAWNS: i32 = 4; /// Max monsters per room

/// Fills a room with stuff!
pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room: &Rect, map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let mut possible_targets: Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(_map: &Map, rng: &mut RandomNumberGenerator, area: &[usize], map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        // Num spawns is a number between 0 and MAX_SPAWNS. The additions
        // make spawn numbers increase with depth, and also mean that sometimes nothing will spawn.
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_SPAWNS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the entities
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    // Attempt to spawn using the rawmaster. If successful, bail early
    let spawn_result = spawn_named_entity(&RAWS.lock().unwrap(), ecs, &spawn.1, SpawnType::AtPosition{x, y});
    if spawn_result.is_some() {
        return;
    }

    if spawn.1 != "None" {
        rltk::console::log(format!("WARNING: We don't know how to spawn '{}'!", spawn.1));
    }
}
fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
