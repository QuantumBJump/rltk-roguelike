use super::{Skill, Skills};

use regex::Regex;

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static!{
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;

    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }

    (n_dice, die_type, die_bonus)
}

/// Calculates the bonus for an attribute, D&D-style
pub fn attr_bonus(value: i32) -> i32 {
    (value-10)/2
}

pub fn player_hp_per_level(fitness:i32) -> i32 {
    10 + attr_bonus(fitness)
}

pub fn player_hp_at_level(fitness:i32, level: i32) -> i32 {
    10 + (player_hp_per_level(fitness) * level)
}

pub fn npc_hp(fitness:i32, level:i32) -> i32 {
    let mut total = 1;
    for _i in 0..level {
        total += i32::max(1, 8 + attr_bonus(fitness));
    }
    total
}

pub fn mana_per_level(intelligence: i32) -> i32 {
    i32::max(1, 4 + attr_bonus(intelligence))
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        skills.skills[&skill]
    } else {
        -4
    }
}
