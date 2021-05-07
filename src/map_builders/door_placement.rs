use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    #[allow()]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<DoorPlacement> {
        #![allow(dead_code)]
        Box::new(DoorPlacement{})
    }

    /// Given a tile index, indicates whether it is possible to place a door in that tile.
    fn door_possible(&self, build_data: &mut BuilderMap, idx: usize) -> bool {
        // Check for other entities - we don't want to spawn a door on top of something!
        let mut blocked = false;
        for spawn in build_data.spawn_list.iter() {
            if spawn.0 == idx { blocked = true; }
        }
        if blocked { return false; }

        let w = build_data.map.width as usize;
        let h = build_data.map.height as usize;
        let x = idx % build_data.map.width as usize;
        let y = idx / build_data.map.width as usize;

        // Check for east-west door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[idx-1] == TileType::Floor) && // There is a floor tile to the left
            (x < w-2 && build_data.map.tiles[idx+1] == TileType::Floor) && // There is a floor tile to the right
            (y > 1 && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall) && // There is a wall north
            (y < h-2 && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall) // There is a wall south
        {
            return true;
        }

        // Check for north-south door possibility
        if build_data.map.tiles[idx] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[idx-1] == TileType::Wall) && // There is a floor tile to the left
            (x < w-2 && build_data.map.tiles[idx+1] == TileType::Wall) && // There is a floor tile to the right
            (y > 1 && build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Floor) && // There is a wall north
            (y < h-2 && build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Floor) // There is a wall south
        {
            return true;
        }

        false
    }

    fn doors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // Avoids nested borrowing
            for hall in halls.iter() {
                if hall.len() == 1 || hall.len() > 3 { // We aren't interested in tiny corridors
                    if self.door_possible(build_data, hall[0]) {
                        build_data.spawn_list.push((hall[0], "Door".to_string()));
                    }
                }
            }
        } else {
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor && self.door_possible(build_data, i) && rng.roll_dice(1, 3) == 1 {
                    build_data.spawn_list.push((i, "Door".to_string()));
                }
            }
        }
    }
}
