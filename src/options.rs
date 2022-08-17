use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;

lazy_static!{
    pub static ref OPTIONS: Mutex<Options> = Mutex::new(Options::new_default());
}

const DEFAULT_OPTIONS: Options = Options {
    keybinds: KeybindType::Vi,
    vis_mapgen: false,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum KeybindType {
    Vi,
    Numpad,
    Wasd
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Options {
    pub keybinds: KeybindType,
    pub vis_mapgen: bool,
}

impl Options {
    pub fn new_default() -> Options {
        DEFAULT_OPTIONS
    }

    pub fn overwrite(&mut self, options: Options) {
        self.keybinds = options.keybinds;
        self.vis_mapgen = options.vis_mapgen;
    }
}

pub fn apply(options: Options) {
    OPTIONS.lock().unwrap().overwrite(options);
}

pub fn apply_default() {
    OPTIONS.lock().unwrap().overwrite(DEFAULT_OPTIONS)
}

pub fn do_options_exist() -> bool {
    Path::new("./options.json").exists()
}

pub fn load_options() {
    let to_load: Options;
    if !do_options_exist() {
        rltk::console::log("No options file found, loading default options");
        to_load = DEFAULT_OPTIONS;
    } else {
        rltk::console::log("Loading options file...");
        let raw_data = fs::read_to_string("./options.json").unwrap();
        let data = serde_json::from_str(&raw_data);
        if let Ok(data) = data {
            to_load = data;
        } else {
            panic!("Failed to load options: {:?}", data);
        }
    }
    OPTIONS.lock().unwrap().overwrite(to_load);
}
