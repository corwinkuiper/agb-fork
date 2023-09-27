use crate::{battle::EnemyAttack, rng};

use super::{EnemyShip, GeneratedAttack};

#[derive(Clone, Copy, Debug)]
pub struct Standard {
    aggresion: i32,
    difficulty: u32,
}

impl Standard {
    pub fn next_move(&mut self) -> Option<GeneratedAttack> {
        if (rng::gen().rem_euclid(1024) as u32) < self.difficulty * 2 {
            Some(GeneratedAttack {
                attack: self.generate_attack(),
                cooldown: generate_cooldown(self.difficulty),
            })
        } else {
            None
        }
    }

    pub fn generate_max_health(&self) -> u32 {
        (5 + self.difficulty as i32 * 2 + rng::default_roll(self.difficulty * 4)) as u32
    }

    pub fn generate_ship(&self) -> EnemyShip {
        if self.aggresion > 0 {
            EnemyShip::Drone
        } else {
            EnemyShip::Piloted
        }
    }

    fn generate_attack(&self) -> EnemyAttack {
        if rng::gen() < self.aggresion {
            EnemyAttack::Shoot(rng::gen().rem_euclid(((self.difficulty + 2) / 3) as i32) as u32 + 1)
        } else if rng::gen() < self.aggresion {
            EnemyAttack::Shield(
                (rng::gen().rem_euclid(((self.difficulty + 4) / 5) as i32) as u32 + 1).min(5),
            )
        } else {
            EnemyAttack::Heal(rng::gen().rem_euclid(((self.difficulty + 1) / 2) as i32) as u32)
        }
    }

    pub fn new(difficulty: u32) -> Self {
        Self {
            aggresion: rng::gen(),
            difficulty,
        }
    }
}

fn generate_cooldown(difficulty: u32) -> u32 {
    rng::gen().rem_euclid((5 * 60 - difficulty as i32 * 10).max(1)) as u32 + 2 * 60
}
