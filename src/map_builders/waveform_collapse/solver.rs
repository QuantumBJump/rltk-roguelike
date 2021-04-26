use super::{Map, MapChunk};
use std::collections::HashSet;

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size: i32,
    chunks: Vec<Option<usize>>,
    chunks_x: usize,
    chunks_y: usize,
    remaining: Vec<(usize, i32)>, // (index, # of neighbours)
    pub possible: bool,
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map: &Map) -> Solver {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining: Vec<(usize, i32)> = Vec::new();
        for i in 0..(chunks_x*chunks_y) {
            remaining.push((i, 0));
        }

        Solver {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining,
            possible: true,
        }
    }

    /// Determines the index of the chunk, counting an entire chunk as one tile
    fn chunk_idx(&self, x: usize, y: usize) -> usize {
        ((y * self.chunks_x) + x) as usize
    }

    /// Counts the number of *existing* chunks surrounding the given chunk
    fn count_neighbours(&self, chunk_x: usize, chunk_y: usize) -> i32 {
        let mut neighbours = 0;

        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x-1, chunk_y);
            match self.chunks[left_idx] {
                None => {}
                Some(_) => {
                    neighbours += 1;
                }
            }
        }

        if chunk_x < self.chunks_x-1 {
            let right_idx = self.chunk_idx(chunk_x+1, chunk_y);
            match self.chunks[right_idx] {
                None => {}
                Some(_) => {
                    neighbours += 1;
                }
            }
        }

        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y-1);
            match self.chunks[up_idx] {
                None => {}
                Some(_) => {
                    neighbours += 1;
                }
            }
        }

        if chunk_y < self.chunks_y-1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y+1);
            match self.chunks[down_idx] {
                None => {}
                Some(_) => {
                    neighbours += 1;
                }
            }
        }

        neighbours
    }

    pub fn iteration(&mut self, map: &mut Map, rng: &mut super::RandomNumberGenerator) -> bool {
        if self.remaining.is_empty() { return true; } // We have completed the map!

        // Populate the neighbour count of each remaining ungenerated chunk
        let mut remain_copy = self.remaining.clone(); // cloning to please borrow checker
        let mut first_run = true;
        for r in remain_copy.iter_mut() {
            // Iterate through all the remaining chunks
            let idx = r.0;
            let chunk_x = idx % self.chunks_x;
            let chunk_y = idx / self.chunks_x;
            let neighbour_count = self.count_neighbours(chunk_x, chunk_y);
            if neighbour_count > 0 { first_run = false; }
            *r = (r.0, neighbour_count); // Update the neighbour count of the remaining chunk
        }
        // Sort our remaining list by number of neighbours, descending
        // so chunk with most neighbours is first
        remain_copy.sort_by(|a, b| b.1.cmp(&a.1));
        self.remaining = remain_copy;

        // Pick a chunk we havent dealt with yet & get its index, removing from remaining list
        let remaining_index = if first_run {
            // If no remaining chunks have neighbours, pick a random chunk
            (rng.roll_dice(1, self.remaining.len() as i32) -1) as usize
        } else {
            0usize
        }; // remaining_index is the chunk we're trying to place
        let chunk_index = self.remaining[remaining_index].0; // chunk_idx is the physical location of the working chunk
        self.remaining.remove(remaining_index);

        // Get the chunk's (x, y) coords
        let chunk_x = chunk_index % self.chunks_x;
        let chunk_y = chunk_index / self.chunks_x;

        let mut neighbours = 0; // Counting neighbours for this specific chunk
        let mut options: Vec<Vec<usize>> = Vec::new(); // options[direction][idx of possible patterns]
        // options is a list of possible patterns this chunk could be in order to satisfy the constraints of neighbours in each direction.

        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x-1, chunk_y);
            match self.chunks[left_idx] {
                None => {}
                Some(nt) => {
                    neighbours += 1;
                    // Add to `options` all patterns compatible with the chunk to our left
                    options.push(self.constraints[nt].compatible_with[3].clone());
                }
            }
        }
        if chunk_x < self.chunks_x-1 {
            let right_idx = self.chunk_idx(chunk_x+1, chunk_y);
            match self.chunks[right_idx] {
                None => {}
                Some(nt) => {
                    neighbours += 1;
                    // Add to `options` all patterns compatible with the chunk to our right
                    options.push(self.constraints[nt].compatible_with[2].clone());
                }
            }
        }
        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y-1);
            match self.chunks[up_idx] {
                None => {}
                Some(nt) => {
                    neighbours += 1;
                    // Add to `options` all patterns compatible with the chunk to our north
                    options.push(self.constraints[nt].compatible_with[1].clone());
                }
            }
        }
        if chunk_y < self.chunks_y-1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y+1);
            match self.chunks[down_idx] {
                None => {}
                Some(nt) => {
                    neighbours += 1;
                    // Add to `options` all patterns compatible with the chunk to our south
                    options.push(self.constraints[nt].compatible_with[0].clone());
                }
            }
        }

        if neighbours == 0 {
            // There is nothing nearby, so we can choose any pattern!
            let new_chunk_idx = (rng.roll_dice(1, self.constraints.len() as i32)-1) as usize; // Choose a random pattern from our gallery
            self.chunks[chunk_index] = Some(new_chunk_idx); // Record that we've decided what to put here
            // Work out the bounds for where we're placing this chunk
            let left_x = chunk_x as i32 * self.chunk_size as i32;
            let right_x = (chunk_x as i32+1) * self.chunk_size as i32;
            let top_y = chunk_y as i32 * self.chunk_size as i32;
            let bottom_y = (chunk_y as i32 + 1) * self.chunk_size as i32;

            let mut i: usize = 0;
            // Copy the pattern into this chunk
            for y in top_y..bottom_y {
                for x in left_x..right_x {
                    let mapidx = map.xy_idx(x, y);
                    let tile = self.constraints[new_chunk_idx].pattern[i];
                    map.tiles[mapidx] = tile;
                    i += 1;
                }
            }
        } else {
            // There are neighbours, so we try to be compatible with them
            // Create a HashSet from *all* of our options compatible with *any* direction
            // So options_to_check is all patterns which *might* be valid to place here
            let mut options_to_check: HashSet<usize> = HashSet::new();
            for o in options.iter() {
                for i in o.iter() {
                    options_to_check.insert(*i);
                }
            }

            // Now filter down the options_to_check by looking at them one by one and working out if
            // they are *actually* valid.
            let mut possible_options: Vec<usize> = Vec::new();
            for new_chunk_idx in options_to_check.iter() {
                // For each possible option in our set...
                let mut possible = true;
                for o in options.iter() {
                    // If any direction is incompatible with that option, remove it from consideration
                    if !o.contains(new_chunk_idx) { possible = false; }
                }
                if possible {
                    possible_options.push(*new_chunk_idx); // If a pattern is compatible with all its neighbours, add it to the filtered list
                }
            }

            if possible_options.is_empty() {
                // If there are no valid patterns, we've failed.
                rltk::console::log("Oh no! It's not possible!");
                self.possible = false;
                return true;
            } else {
                // If there's only one valid pattern, use it. Otherwise, choose a random valid pattern.
                let new_chunk_idx = if possible_options.len() == 1 { 0 }
                    else { rng.roll_dice(1, possible_options.len() as i32) -1 };
                self.chunks[chunk_index] = Some(new_chunk_idx as usize); // Mark what we've chosen to put there

                // Work out bounds
                let left_x = chunk_x as i32 * self.chunk_size as i32;
                let right_x = (chunk_x as i32+1) * self.chunk_size as i32;
                let top_y = chunk_y as i32 * self.chunk_size as i32;
                let bottom_y = (chunk_y as i32 + 1) * self.chunk_size as i32;

                // Copy in the pattern
                let mut i: usize = 0;
                for y in top_y..bottom_y {
                    for x in left_x..right_x {
                        let mapidx = map.xy_idx(x, y);
                        let tile = self.constraints[new_chunk_idx as usize].pattern[i];
                        map.tiles[mapidx] = tile;
                        i += 1;
                    }
                }
            }
        }

        // We haven't yet finished, run the function again
        false
    }
}
