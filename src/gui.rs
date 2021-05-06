use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use super::{
    CombatStats, Player, GameLog, Name, Map, Position, State, InBackpack,
    Viewshed, RunState, Equipped, HungerClock, HungerState, Hidden,
    rex_assets::RexAssets,
};

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    // Draw HP bar
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    for (_player, stats, hc) in (&players, &combat_stats, &hunger).join() {
        let health = format!("HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));

        match hc.state {
            HungerState::WellFed => ctx.print_color(71, 42, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Well Fed"),
            HungerState::Normal => {},
            HungerState::Hungry => ctx.print_color(71, 42, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "Hungry"),
            HungerState::Starving => ctx.print_color(71, 42, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "Starving"),
        }
    }

    // Draw depth
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(2, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &depth);

    // Draw recent gamelog
    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 { ctx.print(2, y, s); }
        y += 1;
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk, target: Option<(i32, i32)>) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let target_pos;
    if let Some(target) = target {
        target_pos = target;
    } else {
        target_pos = ctx.mouse_pos();
    }
    // Ignore if mouse is out of bounds.
    if target_pos.0 >= map.width || target_pos.1 >= map.height { return; }

    let mut tooltip: Vec<String> = Vec::new();
    // Populate tooltip
    for (name, position, _hidden) in (&names, &positions, !&hidden).join() {
        // Get the indices of all named entities
        let idx = map.xy_idx(position.x, position.y);
        if position.x == target_pos.0 && position.y == target_pos.1 && map.visible_tiles[idx] {
            // If mouse is over entity, and entity is visible
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        // Width is equal to the longest item in the tooltip, plus 3 chars
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if target_pos.0 > 40 {
            // If mouse in right half of screen, render tooltip to left.
            let arrow_pos = Point::new(target_pos.0 - 2, target_pos.1);
            let left_x = target_pos.0 - width;
            let mut y = target_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = width - s.len() as i32;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                }
                y+=1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());
        } else {
            // If mouse in left half of screen, render tooltip to right
            let arrow_pos = Point::new(target_pos.0 + 1, target_pos.1);
            let left_x = target_pos.0 + 3;
            let mut y = target_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                ctx.print_color(arrow_pos.x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), "   ".to_string());
                let padding = (width - s.len() as i32) - 3;
                for i in 0..padding {
                    ctx.print_color(left_x + s.len() as i32 + i + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), " ".to_string());
                }
                y+=1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
        }
    }

}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // Inventory is all items which are in a backpack, where the owner of that item is the player.
    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count(); // Number of items in player's inventory.

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(18, y+count as i32+1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut usable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        usable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(usable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - count / 2) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop what?");
    ctx.print_color(18, y+count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut droppable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        droppable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return(ItemMenuResult::Selected, Some(droppable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

/// Shows a menu to allow the player to remove an equipped item and place it in their backpack.
pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped = gs.ecs.write_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&equipped, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    // Draw box
    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Remove Which Item?");
    ctx.print_color(18, y+count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &equipped, &names).join().filter(|item| item.1.owner == *player_entity ) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Select Target: ");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() { if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 { valid_target = true; }}
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos.0, mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None)
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone)]
pub enum FreeTargetSelection {
    NoResponse,
    Cancel,
    Move{ x: i32, y: i32 }}
pub fn free_target(gs: &mut State, ctx: &mut Rltk, aim: Option<(i32, i32)>) -> FreeTargetSelection {
    let mut aim_tile: (i32, i32);
    let map = gs.ecs.fetch::<Map>();
    let player_pos = gs.ecs.fetch::<Point>();
    if let Some(aim) = aim {
        aim_tile = aim;
    } else {
        aim_tile = (player_pos.x, player_pos.y);
    }
    if gs.mouse_targetting {
        aim_tile = ctx.mouse_pos();
    }
    if aim_tile.0 < 0 { aim_tile.0 = 0; }
    if aim_tile.0 > map.width - 1 { aim_tile.0 = map.width - 1; }
    if aim_tile.1 < 0 { aim_tile.1 = 0; }
    if aim_tile.1 > map.height - 1 { aim_tile.1 = map.height - 1; }
    ctx.print_color(5, 0, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Select Target: ");

    // Draw cursor
    ctx.set_bg(aim_tile.0, aim_tile.1, RGB::named(rltk::CYAN));
    draw_tooltips(&gs.ecs, ctx, Some(aim_tile));

    match ctx.key {
        None => FreeTargetSelection::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => FreeTargetSelection::Cancel,
                // Move target
                VirtualKeyCode::Numpad7 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 - 1,
                        y: aim_tile.1 - 1,
                    };
                },
                VirtualKeyCode::Up |
                VirtualKeyCode::Numpad8 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0,
                        y: aim_tile.1 - 1
                    };
                },
                VirtualKeyCode::Numpad9 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 + 1,
                        y: aim_tile.1 - 1,
                    };
                },
                VirtualKeyCode::Numpad4 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 - 1,
                        y: aim_tile.1,
                    };
                },
                VirtualKeyCode::Numpad6 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 + 1,
                        y: aim_tile.1,
                    };
                },
                VirtualKeyCode::Numpad1 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 - 1,
                        y: aim_tile.1 + 1,
                    };
                },
                VirtualKeyCode::Numpad2 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0,
                        y: aim_tile.1 + 1,
                    };
                },
                VirtualKeyCode::Numpad3 => {
                    gs.mouse_targetting = false;
                    return FreeTargetSelection::Move{
                        x: aim_tile.0 + 1,
                        y: aim_tile.1 + 1,
                    };
                },
                // Toggle mouse targetting
                VirtualKeyCode::Numpad5 => {
                    gs.mouse_targetting = !gs.mouse_targetting;
                    return FreeTargetSelection::NoResponse;
                }
                _ => FreeTargetSelection::NoResponse
            }
        }
    }
}

// Main menu code
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection{ selected: MainMenuSelection },
    Selected{ selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(24, 18, 31, 10, RGB::named(rltk::WHEAT), RGB::named(rltk::BLACK));
    ctx.print_color_centered(20, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rustlike tutorial");
    ctx.print_color_centered(21, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "by Quinn Stevens");

    let mut y = 24;
    if let RunState::MainMenu{ menu_selection: selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color(32, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin [");
            ctx.print_color(39, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "N");
            ctx.print_color(40, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "]ew Game");
        } else {
            ctx.print_color(32, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin [");
            ctx.print_color(39, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "N");
            ctx.print_color(40, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "]ew Game");
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color(35, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "[");
                ctx.print_color(36, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "L");
                ctx.print_color(37, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "]oad Game");
            } else {
                ctx.print_color(35, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "[");
                ctx.print_color(36, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "L");
                ctx.print_color(37, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "]oad Game");
            }
            y += 1;
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color(37, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "[");
            ctx.print_color(38, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Q");
            ctx.print_color(39, y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "]uit");
        } else {
            ctx.print_color(37, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "[");
            ctx.print_color(38, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Q");
            ctx.print_color(39, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "]uit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit }}
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            // Skip Load Game if no save exists.
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection };
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            // Skip Load Game if no save exists.
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection };
                    }
                    VirtualKeyCode::Return |
                    VirtualKeyCode::Space => return MainMenuResult::Selected{ selected: selection },
                    // Direct choices
                    VirtualKeyCode::N => return MainMenuResult::Selected{ selected: MainMenuSelection::NewGame },
                    VirtualKeyCode::L => {
                        if save_exists {
                            return MainMenuResult::Selected{ selected: MainMenuSelection::LoadGame };
                        } else {
                            return MainMenuResult::NoSelection{ selected: selection };
                        }
                    },
                    VirtualKeyCode::Q => return MainMenuResult::Selected{ selected: MainMenuSelection::Quit },
                    // Default
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }
    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult { NoSelection, QuitToMenu }

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Your journey has ended!");
    ctx.print_color_centered(16, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "One day, we'll tell you about how you did...");
    ctx.print_color_centered(17, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "But sadly, that day is not today.");

    ctx.print_color_centered(20, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Press ENTER or ESC to return to the menu");
    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape |
                VirtualKeyCode::Return => {
                    GameOverResult::QuitToMenu
                },
                _ => {GameOverResult::NoSelection}
            }
        }
    }
}
