rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

mod item_structs;
use item_structs::*;
mod mob_structs;
use mob_structs::*;
mod rawmaster;
pub use rawmaster::*;
use std::sync::Mutex;
use serde::{Deserialize};

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder: Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");
    RAWS.lock().unwrap().load(decoder);
}