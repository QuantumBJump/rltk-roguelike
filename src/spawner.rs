use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, Monster,
    BlocksTile, Rect, map::MAPWIDTH, Item, Consumable, ProvidesHealing,
    Ranged, InflictsDamage, AreaOfEffect, Stunned, SerializeMe,
    random_table::RandomTable, Equippable, EquipmentSlot, MeleePowerBonus,
    DefenseBonus, HungerClock, HungerState, ProvidesFood, MagicMapper, Hidden,
    EntryTrigger, SingleActivation,
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
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_SPAWNS + 3) + (map_depth - 1) - 3; // Results w/  MAX_SPAWNS=4: 0, 0, 0, 1, 2, 3, 4

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                // Choose random point in the room
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    // If not already spawning a monster there, add a new monster
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the entities
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Stun Scroll" => stun_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Dagger" => dagger(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            "Longsword" => longsword(ecs, x, y),
            "Tower Shield" => tower_shield(ecs, x, y),
            "Rations" => rations(ecs, x, y),
            "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
            "Bear Trap" => bear_trap(ecs, x, y),
            _ => {}
        }
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
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 2 + (map_depth / 2))
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats{
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y})
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{name: "Health Potion".to_string()})
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{heal_amount: 8})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Magic Missile Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Fireball Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 20 })
        .with(AreaOfEffect{ radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn stun_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Stun Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(Stunned{ turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN3),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Scroll of Magic Mapping".to_string() })
        .with(Item{})
        .with(MagicMapper{})
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// Equipment
fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Dagger".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Longsword".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Tower Shield".to_string() })
        .with(Item{})
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn rations(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Rations".to_string() })
        .with(Item{})
        .with(ProvidesFood{})
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// Traps
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name: "Bear Trap".to_string() })
        .with(Hidden{})
        .with(EntryTrigger{})
        .with(InflictsDamage{ damage: 6 })
        .with(SingleActivation{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
