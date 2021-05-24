use std::collections::{HashMap, HashSet};
use specs::prelude::*;
use crate::components::*;
use super::{Raws};
use specs::saveload::{MarkedBuilder, SimpleMarker};
use crate::random_table::{RandomTable};

/// How to choose where to spawn an entity
/// * `AtPosition{x, y}` - Spawns the entity at tile (x, y)
pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl RawMaster {
    /// Creates a new empty RawMaster
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws{ items: Vec::new(), mobs: Vec::new(), props: Vec::new(), spawn_table: Vec::new() },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        let mut used_names: HashSet<String> = HashSet::new();
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!("WARNING - duplicate item name in raws [{}]", item.name));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        self.mob_index = HashMap::new();
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!("WARNING - duplicate mob name in raws [{}]", mob.name));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        self.prop_index = HashMap::new();
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!("WARNING - duplicate prop name in raws [{}]", prop.name));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!("WARNING - Spawn tables reference unspecified entity {}", spawn.name));
            }
        }
    }

}

/// Spawns a given entity at a given location
fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition{x, y} => {
            eb = eb.with(Position{x, y});
        }
    }

    eb
}

/// Given the json definition of a renderable component, returns that component to be added to an entity.
fn get_renderable_component(renderable: &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order
    }
}

/// Spawns the named item
/// 
/// # Arguments
/// 
/// * `raws` - The rawmaster containing the definitions of spawnable entities
/// * `new_entity` - the entity object to attach components to (usually a newly created entity)
/// * `name` - The name of the entity to spawn, e.g. "Tower Shield", "Healing Potion"
/// * `pos` - How to choose where to spawn the entity.
/// 
/// # Returns
/// `Option<Entity>` - If the rawmaster contains an entity matching the name given in `key`, the return value will be that entity.
/// If no match is found, `None` is returned instead.
pub fn spawn_named_item(raws: &RawMaster, new_entity: EntityBuilder, name: &str, pos: SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(name) {
        // If the given key exists in the rawmaster, set the template equal to that item's raw definition
        let item_template = &raws.raws.items[raws.item_index[name]];

        // Create a builder
        let mut eb = new_entity;
        eb = eb.marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // If the item is renderable, add the renderable component
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        // Give the entity a name
        eb = eb.with(Name{ name: item_template.name.clone() });

        eb = eb.with(crate::components::Item{});

        // If the item is consumable, add the various consumable effects to the item
        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(crate::components::Consumable{});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => {
                        eb = eb.with(ProvidesHealing{ heal_amount: effect.1.parse::<i32>().unwrap() });
                    },
                    "ranged" => { eb = eb.with(Ranged{ range: effect.1.parse::<i32>().unwrap() })},
                    "damage" => { eb = eb.with(InflictsDamage{ damage: effect.1.parse::<i32>().unwrap() }) },
                    "area_of_effect" => { eb = eb.with(AreaOfEffect{ radius: effect.1.parse::<i32>().unwrap() }) },
                    "stunned" => { eb = eb.with(Stunned{ turns: effect.1.parse::<i32>().unwrap() }) },
                    "magic_mapping" => { eb = eb.with(MagicMapper{})},
                    "food" => { eb = eb.with(ProvidesFood{})},
                    _ => {
                        rltk::console::log(format!("Warning: consumable effect {} not implemented.", effect_name));
                    }
                }
            }
        }

        // If the item is a weapon, add that component
        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable{ slot: EquipmentSlot::Melee });
            eb = eb.with(MeleePowerBonus{ power: weapon.power_bonus });
        }
        if let Some(shield) = &item_template.shield {
            eb = eb.with(Equippable{ slot: EquipmentSlot::Shield });
            eb = eb.with(DefenseBonus{ defense: shield.defense_bonus });
        }

        return Some(eb.build());
    }

    None
}

/// Spawns a named mob
/// # Arguments
/// 
/// * `raws` - The rawmaster containing the definitions of spawnable entities
/// * `new_entity` - the entity object to attach components to (usually a newly created entity)
/// * `name` - The name of the entity to spawn, e.g. "Orc", "Goblin"
/// * `pos` - How to choose where to spawn the entity.
/// 
/// # Returns
/// `Option<Entity>` - If the rawmaster contains an entity matching the `name` given, the return value will be that entity.
/// If no match is found, `None` is returned instead.
pub fn spawn_named_mob(raws: &RawMaster, new_entity: EntityBuilder, name: &str, pos: SpawnType) -> Option<Entity> {
    if raws.mob_index.contains_key(name) {
        let mob_template = &raws.raws.mobs[raws.mob_index[name]];

        let mut eb = new_entity;
        eb = eb.marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name{ name: mob_template.name.clone() });

        match mob_template.ai.as_ref() {
            "melee" => eb = eb.with(Monster{}),
            "bystander" => eb = eb.with(Bystander{}),
            "vendor" => eb = eb.with(Vendor{}),
            _ => {}
        }
        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile{});
        }
        eb = eb.with(CombatStats{
            max_hp: mob_template.stats.max_hp,
            hp: mob_template.stats.hp,
            power: mob_template.stats.power,
            defense: mob_template.stats.defense
        });
        // If the mob has a memory, give it the RemembersPlayer component
        if let Some(memory) = &mob_template.memory {
            eb = eb.with(RemembersPlayer{
                max_memory: memory.max_memory,
                memory: 0
            })
        }
        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips{
                available: quips.clone()
            });
        }
        eb = eb.with(Viewshed{ visible_tiles: Vec::new(), range: mob_template.vision_range, dirty: true });

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, new_entity: EntityBuilder, name: &str, pos: SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(name) {
        let prop_template = &raws.raws.props[raws.prop_index[name]];

        let mut eb = new_entity;
        eb = eb.marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        eb = spawn_position(pos, eb);

        // Renderable
        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name{name: prop_template.name.clone() });

        // Is the prop hidden?
        if let Some(hidden) = prop_template.hidden {
            if hidden { eb = eb.with(Hidden{}) };
        }
        // Does it block movement?
        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile { eb = eb.with(BlocksTile{})};
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility { eb = eb.with(BlocksVisibility{})};
        }
        if let Some(door_open) = prop_template.door_open {
            eb = eb.with(Door{ open: door_open })
        }
        if let Some(entry_trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger{});
            for effect in entry_trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => { eb = eb.with(InflictsDamage{ damage: effect.1.parse::<i32>().unwrap()})},
                    "single_activation" => { eb = eb.with(SingleActivation{}) },
                    _ => {}
                }
            }
        }

        return Some(eb.build());
    }
    None
}

/// Spawns a named entity
pub fn spawn_named_entity(raws: &RawMaster, new_entity: EntityBuilder, name: &str, pos: SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(name) {
        return spawn_named_item(raws, new_entity, name, pos);
    } else if raws.mob_index.contains_key(name) {
        return spawn_named_mob(raws, new_entity, name, pos);
    } else if raws.prop_index.contains_key(name) {
        return spawn_named_prop(raws, new_entity, name, pos);
    }

    None
}

/// Gets a raw-defined spawn table for a given depth of the dungeon.
pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = raws.raws.spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}
