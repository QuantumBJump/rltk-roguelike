use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use super::{
    Pools, GameLog, Name, Map, Position, State, InBackpack,
    Viewshed, RunState, Equipped, HungerClock, HungerState, Hidden,
    rex_assets::RexAssets, camera, Attributes, Attribute, Consumable
};

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;
    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy+1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;
    let box_grey: RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    // Draw ui boxes
    draw_hollow_box(ctx, 0, 0, 79, 59, box_grey, black); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_grey, black); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_grey, black); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 8, box_grey, black); // Top-right panel
    // Add box connectors to make it look smoother
    ctx.set(0, 45, box_grey, black, to_cp437('├'));
    ctx.set(49, 8, box_grey, black, to_cp437('├'));
    ctx.set(49, 0, box_grey, black, to_cp437('┬'));
    ctx.set(49, 45, box_grey, black, to_cp437('┴'));
    ctx.set(79, 8, box_grey, black, to_cp437('┤'));
    ctx.set(79, 45, box_grey, black, to_cp437('┤'));

    // Draw the map name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 2;
    let x_pos = (22 - (name_length / 2)) as i32;
    ctx.set(x_pos, 0, box_grey, black, to_cp437('┤'));
    ctx.set(x_pos + name_length as i32, 0, box_grey, black, to_cp437('├'));
    ctx.print_color(x_pos+1, 0, white, black, &map.name);
    std::mem::drop(map);

    // Draw stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!("Health: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("Mana: {}/{}", player_pools.mana.current, player_pools.mana.max);
    let xp = format!("Level: {}", player_pools.level);
    let xp_level_start = (player_pools.level - 1) * 1000;


    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.print_color(50, 3, white, black, &xp);
    ctx.draw_bar_horizontal(64, 1, 14, player_pools.hit_points.current, player_pools.hit_points.max, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    ctx.draw_bar_horizontal(64, 2, 14, player_pools.mana.current, player_pools.mana.max, RGB::named(rltk::BLUE), RGB::named(rltk::BLACK));
    ctx.draw_bar_horizontal(64, 3, 14, player_pools.xp - xp_level_start, 1000, RGB::named(rltk::GOLD), RGB::named(rltk::BLACK));

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);

    // Equipped items
    let mut y = 9;
    let equipped = ecs.read_storage::<Equipped>();
    let name = ecs.read_storage::<Name>();
    for (equipped_by, item_name) in (&equipped, &name).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(50, y, white, black, &item_name.name);
            y += 1;
        }
    }

    // Consumables
    y += 1;
    let green = RGB::from_f32(0.0, 1.0, 0.0);
    let yellow = RGB::named(rltk::YELLOW);
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (carried_by, _consumable, item_name) in (&backpack, &consumables, &name).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(50, y, yellow, black, &format!("↑{}", index));
            ctx.print_color(53, y, green, black, &item_name.name);
            y += 1;
            index += 1;
        }
    }

    // Status effects
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state{
        HungerState::WellFed => ctx.print_color(50, 44, green, black, "Well Fed"),
        HungerState::Normal => {},
        HungerState::Hungry => ctx.print_color(50, 44, RGB::named(rltk::ORANGE), black, "Hungry"),
        HungerState::Starving => ctx.print_color(50, 44, RGB::named(rltk::RED), black, "Starving"),
    }

    // Draw log
    let log = ecs.fetch::<GameLog>();
    let mut y = 46;
    for s in log.entries.iter().rev() {
        if y < 59 { ctx.print(2, y, s); }
        y += 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));

    // Draw tooltips
    draw_tooltips(ecs, ctx);
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_grey: RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_grey, black, name);
    let color: RGB =
        if attribute.modifiers < 0 { RGB::from_f32(1.0, 0.0, 0.0) }
        else if attribute.modifiers == 0 { RGB::named(rltk::WHITE) }
        else { RGB::from_f32(0.0, 1.0, 0.0) };
    ctx.print_color(67, y, color, black, &format!("{}", attribute.base + attribute.modifiers));
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 { ctx.set(72, y, color, black, rltk::to_cp437('+')); }
}


struct Tooltip {
    lines : Vec<String>
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S:ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 { self.lines.len() as i32 + 2i32 }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        let box_grey: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_grey: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        ctx.draw_box(x, y, self.width()-1, self.height()-1, white, box_grey);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_grey };
            ctx.print_color(x+1, y+i as i32+1, col, black, &s);
        }
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();

    let mouse_pos = ctx.mouse_pos();
    let mut target_map_pos = mouse_pos;
    target_map_pos.0 += min_x;
    target_map_pos.1 += min_y;

    // Ignore if mouse is out of bounds.
    if target_map_pos.0 >= map.width-1 ||
        target_map_pos.1 >= map.height-1 ||
        target_map_pos.0 < 1 ||
        target_map_pos.1 < 1
    { return; }
    if !map.visible_tiles[map.xy_idx(target_map_pos.0, target_map_pos.1)] { return; }

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for (entity, name, position, _hidden) in (&entities, &names, &positions, !&hidden).join() {
        if position.x == target_map_pos.0 && position.y == target_map_pos.1 {
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

            // Comment on the entity's attributes
            let attr = attributes.get(entity);
            if let Some(attr) = attr {
                let mut s = "".to_string();
                if attr.might.bonus < 0 { s += "Weak. " };
                if attr.might.bonus > 0 { s += "Strong. " };
                if attr.quickness.bonus < 0 { s += "Clumsy. " };
                if attr.quickness.bonus > 0 { s += "Agile. " };
                if attr.fitness.bonus < 0 { s += "Frail. " };
                if attr.fitness.bonus > 0 { s += "Sturdy. " };
                if attr.intelligence.bonus < 0 { s += "Foolish. " };
                if attr.intelligence.bonus > 0 { s += "Learned. " };
                if s.is_empty() {
                    s = "Quite Average".to_string();
                }
                tip.add(s);
            }

            // Comment on pools
            let stat = pools.get(entity);
            if let Some(stat) = stat {
                tip.add(format!("Level: {}", stat.level));
                if stat.hit_points.current <= stat.hit_points.max / 2 {
                    tip.add("Bloodied".to_string());
                }
            }

            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() { return; }

    let box_grey: RGB = RGB::from_hex("#999999").expect("Oops");
    let white = RGB::named(rltk::WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 > 40 {
        // Render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    ctx.set(arrow_x, arrow_y, white, box_grey, arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height/2) > 50 {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 > 40 {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + 2
        };
        tt.render(ctx, x, y);
        y += tt.height();
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
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
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
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1 && screen_x < (max_x - min_x) &&
                    screen_y > 1 && screen_y < (max_y - min_y)
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    let mut valid_target = false;
    for idx in available_cells.iter() { if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 { valid_target = true; }}
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None)
        }
        match ctx.key {
            None => {}
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => return(ItemMenuResult::Cancel, None),
                    _ => {}
                }
            }
        }
    }

    (ItemMenuResult::NoResponse, None)
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
