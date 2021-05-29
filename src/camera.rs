use specs::prelude::*;
use super::{Map, Position, Renderable, Hidden};
use rltk::{Point, Rltk, RGB};
use crate::map::tile_glyph;

const SHOW_BOUNDARIES: bool = true;
const CONSTRAIN_CAMERA: bool = true;

pub fn get_screen_bounds(ecs: &World, _ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = (48, 44);

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let mut min_x = player_pos.x - center_x;
    let mut max_x = min_x + x_chars as i32;
    let mut min_y = player_pos.y - center_y;
    let mut max_y = min_y + y_chars as i32;

    if CONSTRAIN_CAMERA {
        // Don't let the camera stray outside the bounds of the map
        let map = ecs.fetch::<Map>();
        if max_x > map.width {
            let correction = max_x - map.width;
            min_x -= correction;
            max_x -= correction;
        }
        if max_y > map.height {
            let correction = max_y - map.height;
            min_y -= correction;
            max_y -= correction;
        }
        if min_x < 0 {
            max_x += i32::abs(min_x);
            min_x = 0;
        }
        if min_y < 0 {
            let correction = i32::abs(min_y);
            max_y += correction;
            min_y = 0;
        }
    }

    (min_x, max_x, min_y, max_y)
}
pub fn render_camera(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    // Render the map
    let map_width = map.width-1;
    let map_height = map.height-1;

    let mut y = 0;
    for ty in min_y .. max_y {
        let mut x = 0;
        for tx in min_x .. max_x {
            if tx >= 0 && tx <= map_width && ty >= 0 && ty <= map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(rltk::GREY), RGB::named(rltk::BLACK), rltk::to_cp437('·'));
            }
            x += 1;
        }
        y += 1;
    }

    // Render the entities
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let map = ecs.fetch::<Map>();

    let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render, _hidden) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x >= 0 && entity_screen_x <= map_width && entity_screen_y >= 0 && entity_screen_y <= map_height {
                ctx.set(entity_screen_x, entity_screen_y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

pub fn render_debug_map(map: &Map, ctx: &mut Rltk) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size(); // size of the viewing port in characters

    let center_x = (x_chars / 2) as i32; // half the width of the viewport
    let center_y = (y_chars / 2) as i32; // half the height of the viewport

    let min_x = player_pos.x - center_x; // leftmost X of viewport
    let max_x = min_x + x_chars as i32; // rightmost X of viewport
    let min_y = player_pos.y - center_y; // topmost Y of viewport
    let max_y = min_y + y_chars as i32; // bottommost Y of viewport

    let map_width = map.width-1;
    let map_height = map.height-1;

    let mut y = 0;
    for ty in min_y .. max_y {
        let mut x = 0;
        for tx in min_x .. max_x {
            // iterate across every tile in the map
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                // If the tile is not on the edge of the map...
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                // If we're showing the boundaries of the map, render an interpunct
                ctx.set(x, y, RGB::named(rltk::GREY), RGB::named(rltk::BLACK), rltk::to_cp437('·'));
            }
            x += 1;
        }
        y +=1;
    }
}
