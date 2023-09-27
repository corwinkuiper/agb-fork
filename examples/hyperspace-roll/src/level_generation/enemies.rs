use crate::battle::EnemyAttack;

mod standard;

#[derive(Debug)]
pub enum EnemyAi {
    Standard(standard::Standard),
}

pub struct GeneratedAttack {
    pub attack: EnemyAttack,
    pub cooldown: u32,
}

pub enum EnemyShip {
    Drone,
    Piloted,
}

impl EnemyAi {
    pub fn next_move(&mut self) -> Option<GeneratedAttack> {
        match self {
            EnemyAi::Standard(standard) => standard.next_move(),
        }
    }

    pub fn generate_ship(&self) -> EnemyShip {
        match self {
            EnemyAi::Standard(standard) => standard.generate_ship(),
        }
    }

    pub fn generate_max_health(&self) -> u32 {
        match self {
            EnemyAi::Standard(standard) => standard.generate_max_health(),
        }
    }

    pub fn generate(current_level: u32) -> EnemyAi {
        EnemyAi::Standard(standard::Standard::new(current_level))
    }
}
