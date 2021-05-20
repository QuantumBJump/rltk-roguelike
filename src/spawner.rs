use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, Rect,
    SerializeMe, random_table::RandomTable, HungerClock, HungerState, Map,
    TileType, raws::*,
};
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns their entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
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
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(HungerClock{
            state: HungerState::WellFed,
            duration: 20,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
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
    let spawn_result = spawn_named_entity(&RAWS.lock().unwrap(), ecs.create_entity(), &spawn.1, SpawnType::AtPosition{x, y});
    if spawn_result.is_some() {
        return;
    }

    if spawn.1 != "None" {
        rltk::console::log(format!("WARNING: We don't know how to spawn '{}'!", spawn.1));
    }
}
fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Stun Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Buckler", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 2 + (map_depth / 2))
}
