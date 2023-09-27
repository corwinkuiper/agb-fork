use agb::{hash_map::HashMap, rng};
use alloc::vec::Vec;

mod enemies;

pub use enemies::{EnemyAi, EnemyShip, GeneratedAttack};

use crate::Face;

pub fn generate_upgrades(level: u32, call: &mut dyn FnMut()) -> Vec<Face> {
    let mut upgrade_values = HashMap::new();

    upgrade_values.insert(Face::Shoot, 5);
    upgrade_values.insert(Face::DoubleShot, 10);
    upgrade_values.insert(Face::DoubleShotValue, 15);
    upgrade_values.insert(Face::TripleShot, 20);
    upgrade_values.insert(Face::TripleShotValue, 30);
    upgrade_values.insert(Face::Shield, 5);
    upgrade_values.insert(Face::DoubleShield, 10);
    upgrade_values.insert(Face::TripleShield, 20);
    upgrade_values.insert(Face::DoubleShieldValue, 25);
    upgrade_values.insert(Face::Malfunction, -2);
    upgrade_values.insert(Face::Bypass, 7);
    upgrade_values.insert(Face::Disrupt, 10);
    upgrade_values.insert(Face::MalfunctionShot, 15);
    upgrade_values.insert(Face::Heal, 8);
    upgrade_values.insert(Face::BurstShield, 30);
    upgrade_values.insert(Face::Invert, 30);

    let potential_upgrades: Vec<Face> = upgrade_values.keys().cloned().collect();

    let mut upgrades = Vec::new();

    let upgrade_value = |upgrades: &[Face], potential_upgrade: Face| -> i32 {
        upgrades
            .iter()
            .map(|x| upgrade_values.get(x).unwrap())
            .sum::<i32>()
            + upgrade_values.get(&potential_upgrade).unwrap()
    };

    let max_upgrade_value = 15 + (rng::gen().rem_euclid(level as i32 * 5));
    let mut attempts = 0;

    while upgrades.len() != 3 {
        call();

        attempts += 1;
        let next = potential_upgrades[rng::gen() as usize % potential_upgrades.len()];
        let number_of_malfunctions = upgrades
            .iter()
            .chain(core::iter::once(&next))
            .filter(|&x| *x == Face::Malfunction)
            .count();
        let maximum_number_of_malfunctions = (level >= 5).into();
        if upgrade_value(&upgrades, next) <= max_upgrade_value
            && number_of_malfunctions <= maximum_number_of_malfunctions
        {
            upgrades.push(next);
            attempts = 0;
        }

        if attempts > 100 {
            attempts = 0;
            upgrades.clear();
        }
    }

    upgrades
}
