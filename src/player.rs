use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::{
    Position, Player, State, Map, Viewshed, RunState, Pools,
    WantsToMelee, Item, gamelog::GameLog, WantsToPickupItem, TileType, Monster,
    HungerClock, HungerState, EntityMoved, Door, BlocksVisibility, BlocksTile,
    Renderable, Bystander, Vendor, options::OPTIONS, options::KeybindType,
};
use std::cmp::{min, max};

#[derive(PartialEq, Copy, Clone)]
enum Command {
    Move{x: i32, y: i32},
    Wait,
    Get,
    Inventory,
    Drop,
    Remove,
    Menu,
    Descend,
    Help,
    Undefined,
}

fn key_to_command(key: VirtualKeyCode) -> Command {
    let keybinds = OPTIONS.lock().unwrap().keybinds;
    match keybinds {
        // Match the keys which differ between keybind setups
        KeybindType::Vi => {
            match key {
                VirtualKeyCode::H => return Command::Move{x: -1,y: 0},
                VirtualKeyCode::J => return Command::Move{x: 0, y: 1},
                VirtualKeyCode::K => return Command::Move{x: 0, y: -1},
                VirtualKeyCode::L => return Command::Move{x: 1, y: 0},
                VirtualKeyCode::Y => return Command::Move{x: -1, y: -1},
                VirtualKeyCode::U => return Command::Move{x: 1, y: -1},
                VirtualKeyCode::N => return Command::Move{x: -1, y: 1},
                VirtualKeyCode::M => return Command::Move{x: 1, y: 1},
                VirtualKeyCode::Semicolon => return Command::Wait,
                VirtualKeyCode::D => return Command::Drop,
                _ => {}
            }
        }
        KeybindType::Numpad => {
            match key {
                VirtualKeyCode::Numpad4 => return Command::Move{x: -1,y: 0},
                VirtualKeyCode::Numpad6 => return Command::Move{x: 1, y: 0},
                VirtualKeyCode::Numpad8 => return Command::Move{x: 0, y: -1},
                VirtualKeyCode::Numpad2 => return Command::Move{x: 0, y: 1},
                VirtualKeyCode::Numpad7 => return Command::Move{x: -1, y: -1},
                VirtualKeyCode::Numpad9 => return Command::Move{x: 1, y: -1},
                VirtualKeyCode::Numpad1 => return Command::Move{x: -1, y: 1},
                VirtualKeyCode::Numpad3 => return Command::Move{x: 1, y: 1},
                VirtualKeyCode::Numpad5 => return Command::Wait,
                VirtualKeyCode::D => return Command::Drop,
                _ => {}
            }
        }
        KeybindType::Wasd => {
            match key {
                VirtualKeyCode::A => return Command::Move{x: -1,y: 0},
                VirtualKeyCode::S => return Command::Move{x: 0, y: 1},
                VirtualKeyCode::W => return Command::Move{x: 0, y: -1},
                VirtualKeyCode::D => return Command::Move{x: 1, y: 0},
                VirtualKeyCode::Q => return Command::Move{x: -1, y: -1},
                VirtualKeyCode::E => return Command::Move{x: 1, y: -1},
                VirtualKeyCode::Z => return Command::Move{x: -1, y: 1},
                VirtualKeyCode::C => return Command::Move{x: 1, y: 1},
                VirtualKeyCode::X => return Command::Wait,
                VirtualKeyCode::T => return Command::Drop,
                _ => {}
            }
        }
    }
    match key {
        // Match keycodes which are the same between setups
        VirtualKeyCode::Escape => return Command::Menu,
        VirtualKeyCode::Period => return Command::Descend,
        VirtualKeyCode::G => return Command::Get,
        VirtualKeyCode::I => return Command::Inventory,
        VirtualKeyCode::R => return Command::Remove,
        VirtualKeyCode::Slash => return Command::Help,
        _ => {}
    }
    return Command::Undefined;
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let bystanders = ecs.read_storage::<Bystander>();
    let vendors = ecs.read_storage::<Vendor>();

    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();

    let mut opened_door = false;

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        // Don't let player move out of bounds.
        if pos.x + delta_x < 0 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 0 || pos.y + delta_y > map.height-1 { return; }

        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let bystander = bystanders.get(*potential_target);
            let vendor = vendors.get(*potential_target);
            if bystander.is_some() || vendor.is_some() {
                // Note that we want to move the bystander
                swap_entities.push((*potential_target, pos.x, pos.y));

                // Move the player anyway, even though the space is "blocked"
                pos.x = min(map.width-1, max(0, pos.x + delta_x));
                pos.y = min(map.height-1, max(0, pos.y + delta_y));
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");

                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
            } else {
                let target = combat_stats.get(*potential_target);
                if let Some(_target) = target {
                    wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed.");
                    return; // Don't move after attacking.
                }
            }
            let door = doors.get_mut(*potential_target);
            if let Some(door) = door {
                door.open = true;
                blocks_visibility.remove(*potential_target);
                blocks_movement.remove(*potential_target);
                let glyph = renderables.get_mut(*potential_target).unwrap();
                glyph.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
                opened_door = true;
            }
        }
        if !map.blocked[destination_idx] {
            pos.x = min(map.width-1, max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
        }
    }

    // If we opened a door, update the viewsheds of everything on the map to work out if they can now see through the door.
    if opened_door {
        for v in (&mut viewsheds).join() {
            v.dirty = true;
        }
    }

    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            their_pos.x = m.1;
            their_pos.y = m.2;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // Check if the player is standing on an item & if so, target it.
    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing to pick up here!".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item}).expect("Unable to insert want to pickup");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();
    
    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => {can_heal = false;}
            }
        }
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hc = hunger_clocks.get(*player_entity);
    if let Some(hc) = hc {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        pools.hit_points.current = i32::min(pools.hit_points.current + 1, pools.hit_points.max);
    }

    RunState::PlayerTurn
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack, WantsToUseItem};

    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        use crate::components::Ranged;
        if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(carried_consumables[key as usize]) {
            return RunState::ShowTargeting{ range: ranged.range, item: carried_consumables[key as usize] };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent.insert(
            *player_entity,
            WantsToUseItem{ item: carried_consumables[key as usize], target: None }
        ).expect("Unable to insert intent");
        return RunState::PlayerTurn;

    }

    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Hotkeys
    if ctx.control {
        if ctx.key.is_some() {
            let key: Option<i32> =
                match ctx.key.unwrap() {
                    VirtualKeyCode::Key1 => Some(1),
                    VirtualKeyCode::Key2 => Some(2),
                    VirtualKeyCode::Key3 => Some(3),
                    VirtualKeyCode::Key4 => Some(4),
                    VirtualKeyCode::Key5 => Some(5),
                    VirtualKeyCode::Key6 => Some(6),
                    VirtualKeyCode::Key7 => Some(7),
                    VirtualKeyCode::Key8 => Some(8),
                    VirtualKeyCode::Key9 => Some(9),
                    VirtualKeyCode::Key0 => Some(10),
                    _ => None
                };
            if let Some(key) = key {
                return use_consumable_hotkey(gs, key-1);
            }
        }
    }
    // Player movement
    match ctx.key {
        None => { return RunState::AwaitingInput } // Nothing happened
        Some(key) => {
            let command = key_to_command(key);
            match command {
                // Wait button
                Command::Wait => return skip_turn(&mut gs.ecs),

                // Collect item
                Command::Get => get_item(&mut gs.ecs),

                // Open inventory
                Command::Inventory => return RunState::ShowInventory,

                // Drop item
                Command::Drop => return RunState::ShowDropItem,

                // Remove equipped item
                Command::Remove => return RunState::ShowRemoveItem,

                // Movement
                Command::Move{x, y} => try_move_player(x, y, &mut gs.ecs),

                // Level changes
                Command::Descend => {
                    if try_next_level(&mut gs.ecs) {
                        return RunState::NextLevel;
                    }
                }

                // Menu
                Command::Menu => return RunState::SaveGame,

                // Show help
                Command::Help => return RunState::ShowHelp,

                _ => { return RunState::AwaitingInput } // Key not recognised
            }
        }
    }
    RunState::PlayerTurn
}
