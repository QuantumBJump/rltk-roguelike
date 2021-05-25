use super::{BuilderChain, BuilderMap, InitialMapBuilder, Position, TileType};
use std::collections::HashSet;

enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}

pub fn town_builder(
    new_depth: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height);
    chain.start_with(TownBuilder::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    #![allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder {})
    }

    pub fn build_rooms(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);

        // Make visible for screenshot
        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();

        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        // Place exit
        let exit_idx = build_data.map.xy_idx(build_data.width - 5, wall_gap_y);
        build_data.map.tiles[exit_idx] = TileType::DownStairs;

        // Sort buildings by size (we want the largest building to be the pub)
        let building_size = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_size);

        // Spawn outdoor NPCs
        self.spawn_dockers(build_data, rng);
        self.spawn_townsfolk(build_data, rng, &mut available_building_tiles);
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        // We'll start with a nice layer of grass
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    /// Creates a shoreline on the west side of the map, with piers jutting out over the water.
    fn water_and_piers(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width: Vec<i32> = Vec::new(); // The width of the water at a given y level
        for y in 0..build_data.height {
            // The sin here means the coastline waves in and out
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 3);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water + 3 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add piers
        for _i in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.height) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::WoodFloor;
            }
        }
        build_data.take_snapshot();
    }

    /// Creates the boundaries of a town in which to place buildings.
    ///
    /// Creates a rectangle of walls, with a road running through it from east to west, and gravel
    /// everywhere else inside. The road exists at a random y-height, so it doesn't go through
    /// the same part of town every time.
    ///
    /// # Returns
    /// * `HashSet<usize>`: A set of tile indices showing where it's possible to build within the
    /// town. This essentially corresponds to all the gravel areas.
    /// * `i32`: The y coordinate of the center of the road.
    fn town_walls(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles: HashSet<usize> = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5;
        for y in 1..build_data.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let idx = build_data.map.xy_idx(30, y);
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx - 1] = TileType::Floor;
                let idx_right = build_data.map.xy_idx(build_data.width - 2, y);
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.width - 2 {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.height - 1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.width - 1 {
            let idx_top = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bot = build_data.map.xy_idx(x, build_data.height - 2);
            build_data.map.tiles[idx_bot] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    /// Creates 12 buildings in the town.
    /// The buildings are wood-floored and walled in.
    ///
    /// ### Returns
    /// * `Vec<(i32, i32, i32, i32)>`: A vector of building locations, in the following form:
    /// (top_left_x, top_left_y, width, height)
    fn buildings(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                    } else {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) {
                            possible = false;
                        }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by + bh {
                    for x in bx..bx + bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx); // Don't build on this tile again
                                                               // Also remove adjacent tiles so buildings don't pop up right next to each other.
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx + build_data.width as usize));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx - build_data.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // Outline buildings
        let mut mapclone = build_data.map.clone();
        for y in 2..build_data.height - 2 {
            for x in 31..build_data.width - 2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor {
                    // Count the number of neighbours which are not wood floors
                    let mut neighbours = 0;
                    if build_data.map.tiles[idx - 1] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx + 1] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx + build_data.width as usize] != TileType::WoodFloor
                    {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx - build_data.width as usize] != TileType::WoodFloor
                    {
                        neighbours += 1;
                    }
                    if neighbours > 0 {
                        // If the tile is in contact with the outside at all, turn it into a wall
                        mapclone.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();
        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<(i32, i32, i32, i32)>,
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + building.3 / 2;
            let idx = if cy > wall_gap_y {
                // Door on the north wall
                build_data.map.xy_idx(door_x, building.1)
            } else {
                // Door on south wall
                build_data.map.xy_idx(door_x, building.1 + building.3 - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &[usize]) {
        // Work out where the road is
        let mut roads = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Road {
                    roads.push(idx);
                }
            }
        }

        build_data.map.populate_blocked(); // Populate pathfinding info
        for door_idx in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new(); // (road tile, distance)
            let door_pt = rltk::Point::new(
                *door_idx as i32 % build_data.width,
                *door_idx as i32 / build_data.width,
            );
            for r in roads.iter() {
                let road_pt =
                    rltk::Point::new(*r as i32 % build_data.width, *r as i32 / build_data.width);
                nearest_roads.push((
                    *r,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(door_pt, road_pt),
                ));
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0; // index of closest road tile (as the crow flies)
            let path = rltk::a_star_search(*door_idx, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
            build_data.take_snapshot();
        }
    }

    /// Sorts and tags all the buildings in the town.
    fn sort_buildings(
        &mut self,
        buildings: &[(i32, i32, i32, i32)],
    ) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((
                i,
                building.2 * building.3, // width * height
                BuildingTag::Unassigned
            ));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1)); // Sort buildings descending by size
        // Tag all the buildings
        building_size[0].2 = BuildingTag::Pub;
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Blacksmith;
        building_size[3].2 = BuildingTag::Clothier;
        building_size[4].2 = BuildingTag::Alchemist;
        building_size[5].2 = BuildingTag::PlayerHouse;
        for b in building_size.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }
        let last_index = building_size.len()-1;
        building_size[last_index].2 = BuildingTag::Abandoned;
        building_size.sort_by(|a, b| a.0.cmp(&b.0));

        building_size
    }

    fn building_factory(&mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[(i32, i32, i32, i32)],
        building_index: &[(usize, i32, BuildingTag)],
    ) {
        for (i, building) in buildings.iter().enumerate() {
            let build_type = &building_index[i].2;
            match build_type {
                BuildingTag::Pub => self.build_pub(&building, build_data, rng),
                BuildingTag::Temple => self.build_temple(&building, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(&building, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(&building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(&building, build_data, rng),
                BuildingTag::PlayerHouse => self.build_my_house(&building, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(&building, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(&building, build_data, rng),
                _ => {}
            }
        }
    }

    /// Takes a building and a list of things to spawn in the building, and spawns them in (sort of) random places
    fn random_building_spawn(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_idx: usize,
    ) {
        for y in building.1..building.1 + building.3 {
            for x in building.0..building.0 + building.2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor && idx != player_idx && rng.roll_dice(1, 3)==1 && !to_place.is_empty() {
                    let entity_tag = to_place[0];
                    to_place.remove(0);
                    build_data.spawn_list.push((idx, entity_tag.to_string()));
                }
            }
        }
    }

    /// Builds the pub. Adds some other hung-over patrons, a "lost" goods salesperson, a barkeep,
    /// tables, chairs and barrels.
    fn build_pub(&mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place the player
        build_data.starting_position = Some(Position{
            x: building.0 + (building.2 / 2),
            y: building.1 + (building.3 / 2),
        });
        let player_idx = build_data.map.xy_idx(build_data.starting_position.as_ref().unwrap().x, build_data.starting_position.as_ref().unwrap().y);

        // Place other items
        let mut to_place: Vec<&str> = vec!["Barkeep", "Shady Vendor", "Patron", "Patron", "Table", "Chair", "Table", "Chair"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_idx);
    }

    /// Builds the temple
    ///
    /// Contents:
    /// * Priest(s)
    /// * Parishioners
    /// * Chairs
    /// * Candles
    fn build_temple(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Priest", "Parishioner", "Parishioner", "Chair", "Chair", "Candle", "Candle"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_smith(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Blacksmith", "Anvil", "Water Trough", "Weapon Rack", "Armour Stand"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_clothier(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Clothier", "Cabinet", "Table", "Loom", "Hide Rack"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_alchemist(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Alchemist", "Chemistry Set", "Dead Thing", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_my_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Mum", "Bed", "Cabinet", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_hovel(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let mut to_place: Vec<&str> = vec!["Peasant", "Bed", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_abandoned_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator
    ) {
        let max_rats = 5;
        let mut curr_rats = 0;
        for y in building.1 .. building.1 + building.3 {
            for x in building.0 .. building.0 + building.2 {
                if curr_rats == max_rats {
                    return
                }
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor && idx != 0 && rng.roll_dice(1, 2)==1 {
                    build_data.spawn_list.push((idx, "Rat".to_string()));
                    curr_rats += 1;
                }
            }
        }
    }

    fn spawn_dockers(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator) {
        for (idx, tt) in build_data.map.tiles.iter().enumerate() {
            if *tt == TileType::Bridge && rng.roll_dice(1, 6) == 1 {
                let roll = rng.roll_dice(1, 3);
                match roll {
                    1 => build_data.spawn_list.push((idx, "Dock Worker".to_string())),
                    2 => build_data.spawn_list.push((idx, "Wannabe Pirate".to_string())),
                    _ => build_data.spawn_list.push((idx, "Fisher".to_string())),
                }
            }
        }
    }

    fn spawn_townsfolk(&mut self, build_data: &mut BuilderMap, rng: &mut rltk::RandomNumberGenerator, available_building_tiles: &mut HashSet<usize>) {
        for idx in available_building_tiles.iter() {
            if rng.roll_dice(1, 10) == 1{
                let roll = rng.roll_dice(1, 4);
                match roll {
                    1 => build_data.spawn_list.push((*idx, "Peasant".to_string())),
                    2 => build_data.spawn_list.push((*idx, "Drunk".to_string())),
                    3 => build_data.spawn_list.push((*idx, "Dock Worker".to_string())),
                    _ => build_data.spawn_list.push((*idx, "Fisher".to_string())),
                }
            }
        }
    }
}
