use specs::prelude::*;
use super::{
    Attributes, WantsToMelee, Name, SufferDamage, gamelog::GameLog,
    HungerClock, HungerState, particle_system::ParticleBuilder, Position,
    Skills, Pools, Skill, Equipped, MeleeWeapon, WeaponAttribute, EquipmentSlot,
    Wearable, NaturalAttackDefense
};
use crate::{skill_bonus};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefense>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, mut log, mut wants_melee, names, attributes, skills,
            mut inflict_damage, mut particle_builder, positions, hunger_clock,
            pools, mut rng, equipped_items, meleeweapons, wearables, natural,
            player_entity
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (&entities, &wants_melee, &names, &attributes, &skills, &pools).join() {
            // Are the attacker and defender both alive? Only attack if they are
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

                let mut weapon_info = MeleeWeapon{
                    attribute: WeaponAttribute::Might,
                    hit_bonus: 0,
                    damage_n_dice: 1,
                    damage_die_type: 4,
                    damage_bonus: 0
                };

                if let Some(nat) = natural.get(entity) {
                    if !nat.attacks.is_empty() {
                        // Choose a random natural attack from those available to the monster
                        let attack_index = if nat.attacks.len()==1 { 0 } else { rng.roll_dice(1, nat.attacks.len() as i32) as usize -1 };
                        weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                        weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                        weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                        weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                    }
                }

                for (wielded, melee) in (&equipped_items, &meleeweapons).join() {
                    if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                        weapon_info = melee.clone();
                    }
                }

                // Calculate attack roll
                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = if weapon_info.attribute == WeaponAttribute::Might
                    { attacker_attributes.might.bonus }
                    else { attacker_attributes.quickness.bonus };
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = weapon_info.hit_bonus;
                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(entity) { // Well Fed grants +1
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }
                let modified_hit_roll = natural_roll + attribute_hit_bonus + skill_hit_bonus + status_hit_bonus + weapon_hit_bonus;

                // Calculate defender's AC
                let mut armour_item_bonus_f = 0.0;
                for (wielded, armour) in (&equipped_items, &wearables).join() {
                    if wielded.owner == wants_melee.target {
                        armour_item_bonus_f += armour.armour_class;
                    }
                }
                let base_armour_class = match natural.get(wants_melee.target) {
                    None => 10,
                    Some(nat) => nat.armour_class.unwrap_or(10)
                };
                let armour_quickness_bonus = target_attributes.quickness.bonus;
                let armour_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armour_item_bonus = armour_item_bonus_f as i32;
                let armour_class = base_armour_class + armour_quickness_bonus + armour_skill_bonus + armour_item_bonus;

                // Determine if the attack hits
                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll >= armour_class) {
                    // Target hit! Until we support weapons, we'll just deal 1d4 damage
                    let mut base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    if natural_roll == 20 {
                        log.entries.push("Critical hit!!".to_string());
                        base_damage += rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    }
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(0, base_damage + attr_damage_bonus + skill_hit_bonus + skill_damage_bonus + weapon_damage_bonus);
                    let from_player = entity == *player_entity;
                    SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage, from_player);
                    log.entries.push(format!("{} hits {} for {} damage!", name.name, target_name.name, damage));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                } else if natural_roll == 1 {
                    // Critical miss!
                    log.entries.push(format!("{} attacks {} - critical miss!", name.name, target_name.name));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                } else {
                    // Miss
                    if modified_hit_roll > armour_class - armour_item_bonus {
                        // The attack wouldn't have hit even without armour, so it's a dodge
                        log.entries.push(format!("{} attacks {}, but {} dodges!", name.name, target_name.name, target_name.name));
                    } else {
                        log.entries.push(format!("{} attacks {}, but bounces off the armour!", name.name, target_name.name));
                    }
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}