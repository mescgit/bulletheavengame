use bevy::prelude::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, PartialEq)] 
pub enum UpgradeType {
    PlayerSpeed(u32),
    MaxHealth(i32),
    BulletDamage(i32),
    FireRate(u32),
    ProjectileSpeed(u32),
    ProjectilePiercing(u32), 
    XPGainMultiplier(u32),  
    PickupRadiusIncrease(u32),
    IncreaseProjectileCount(u32),
    // AoE Aura Upgrades
    UnlockAoeAuraWeapon,
    IncreaseAoeAuraRadius(u32), 
    IncreaseAoeAuraDamage(i32), 
    DecreaseAoeAuraTickRate(u32),
    HealthRegeneration(f32), 
    // Orbiting Seeds Upgrades
    UnlockOrbitingSeeds,
    IncreaseOrbiterCount(u32), // Add N orbiters
    IncreaseOrbiterDamage(i32),
    IncreaseOrbiterRadius(f32), // Flat increase to orbit radius
    IncreaseOrbiterRotationSpeed(f32), // Additive to rotation speed (rad/s)
}

#[derive(Debug, Clone)]
pub struct UpgradeCard {
    pub id: UpgradeId,
    pub name: String,
    pub description: String,
    pub upgrade_type: UpgradeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpgradeId(pub u32);

#[derive(Resource, Default)]
pub struct UpgradePool {
    pub available_upgrades: Vec<UpgradeCard>,
}

impl UpgradePool {
    pub fn initialize(&mut self) {
        self.available_upgrades = vec![
            // ... (all previous upgrades) ...
            UpgradeCard { id: UpgradeId(0), name: "Adrenaline Rush".to_string(), description: "Increase player speed by 10%.".to_string(), upgrade_type: UpgradeType::PlayerSpeed(10)},
            UpgradeCard { id: UpgradeId(1), name: "Reinforced Hull".to_string(), description: "Increase Max Health by 20.".to_string(), upgrade_type: UpgradeType::MaxHealth(20)},
            UpgradeCard { id: UpgradeId(2), name: "Potent Seeds".to_string(), description: "Increase Seed damage by 5.".to_string(), upgrade_type: UpgradeType::BulletDamage(5)},
            UpgradeCard { id: UpgradeId(3), name: "Rapid Growth".to_string(), description: "Increase firing speed by 15%.".to_string(), upgrade_type: UpgradeType::FireRate(15)},
            UpgradeCard { id: UpgradeId(4), name: "Aerodynamic Seeds".to_string(), description: "Increase projectile speed by 20%.".to_string(), upgrade_type: UpgradeType::ProjectileSpeed(20)},
            UpgradeCard { id: UpgradeId(5), name: "Fleet Footed".to_string(), description: "Increase player speed by 15%.".to_string(), upgrade_type: UpgradeType::PlayerSpeed(15)},
            UpgradeCard { id: UpgradeId(6), name: "Vitality Boost".to_string(), description: "Increase Max Health by 30.".to_string(), upgrade_type: UpgradeType::MaxHealth(30)},
            UpgradeCard { id: UpgradeId(7), name: "Sharpened Thorns".to_string(), description: "Increase Seed damage by 8.".to_string(), upgrade_type: UpgradeType::BulletDamage(8)},
            UpgradeCard { id: UpgradeId(8), name: "Hyper Growth".to_string(), description: "Increase firing speed by 20%.".to_string(), upgrade_type: UpgradeType::FireRate(20)},
            UpgradeCard { id: UpgradeId(9), name: "Piercing Seeds".to_string(), description: "Seeds pierce +1 enemy.".to_string(), upgrade_type: UpgradeType::ProjectilePiercing(1)},
            UpgradeCard { id: UpgradeId(10), name: "Knowledge Seeker".to_string(), description: "Gain +20% more XP from orbs.".to_string(), upgrade_type: UpgradeType::XPGainMultiplier(20)},
            UpgradeCard { id: UpgradeId(11), name: "Magnetic Aura".to_string(), description: "Increase XP orb attraction radius by 25%.".to_string(), upgrade_type: UpgradeType::PickupRadiusIncrease(25)},
            UpgradeCard { id: UpgradeId(12), name: "Deep Roots".to_string(), description: "Seeds pierce +2 enemies.".to_string(), upgrade_type: UpgradeType::ProjectilePiercing(2)},
            UpgradeCard { id: UpgradeId(13), name: "Enlightenment".to_string(), description: "Gain +30% more XP from orbs.".to_string(), upgrade_type: UpgradeType::XPGainMultiplier(30)},
            UpgradeCard { id: UpgradeId(100), name: "Protective Spores".to_string(), description: "Unlock a damaging aura around you.".to_string(), upgrade_type: UpgradeType::UnlockAoeAuraWeapon},
            UpgradeCard { id: UpgradeId(101), name: "Expanding Spores".to_string(), description: "Increase aura radius by 20%.".to_string(), upgrade_type: UpgradeType::IncreaseAoeAuraRadius(20)},
            UpgradeCard { id: UpgradeId(102), name: "Potent Spores".to_string(), description: "Increase aura damage by 2.".to_string(), upgrade_type: UpgradeType::IncreaseAoeAuraDamage(2)},
            UpgradeCard { id: UpgradeId(103), name: "Rapid Spores".to_string(), description: "Aura damages 15% faster.".to_string(), upgrade_type: UpgradeType::DecreaseAoeAuraTickRate(15)},
            UpgradeCard { id: UpgradeId(200), name: "Twin Seeds".to_string(), description: "Fire +1 additional seed.".to_string(), upgrade_type: UpgradeType::IncreaseProjectileCount(1)},
            UpgradeCard { id: UpgradeId(201), name: "Seed Barrage".to_string(), description: "Fire +2 additional seeds.".to_string(), upgrade_type: UpgradeType::IncreaseProjectileCount(2)},
            UpgradeCard { id: UpgradeId(300), name: "Photosynthesis".to_string(), description: "Regenerate 0.5 HP per second.".to_string(), upgrade_type: UpgradeType::HealthRegeneration(0.5)},
            UpgradeCard { id: UpgradeId(301), name: "Strong Roots".to_string(), description: "Regenerate 1.0 HP per second.".to_string(), upgrade_type: UpgradeType::HealthRegeneration(1.0)},

            // Orbiting Seeds Upgrades
            UpgradeCard {
                id: UpgradeId(400),
                name: "Guardian Seeds".to_string(),
                description: "Summon 2 orbiting seeds that damage enemies.".to_string(),
                upgrade_type: UpgradeType::UnlockOrbitingSeeds,
            },
            UpgradeCard {
                id: UpgradeId(401),
                name: "More Guardians".to_string(),
                description: "Add +1 orbiting seed.".to_string(),
                upgrade_type: UpgradeType::IncreaseOrbiterCount(1),
            },
            UpgradeCard {
                id: UpgradeId(402),
                name: "Sharper Guardians".to_string(),
                description: "Increase orbiting seed damage by 3.".to_string(),
                upgrade_type: UpgradeType::IncreaseOrbiterDamage(3),
            },
            UpgradeCard {
                id: UpgradeId(403),
                name: "Wider Orbit".to_string(),
                description: "Increase orbit radius by 15.".to_string(),
                upgrade_type: UpgradeType::IncreaseOrbiterRadius(15.0),
            },
            UpgradeCard {
                id: UpgradeId(404),
                name: "Faster Orbit".to_string(),
                description: "Increase orbit speed by 0.5 rad/s.".to_string(),
                upgrade_type: UpgradeType::IncreaseOrbiterRotationSpeed(0.5),
            },
        ];
    }

    pub fn get_random_upgrades(&self, count: usize) -> Vec<UpgradeCard> {
        let mut rng = rand::thread_rng();
        self.available_upgrades
            .choose_multiple(&mut rng, count)
            .cloned()
            .collect()
    }
}

#[derive(Component, Debug, Clone)]
pub struct OfferedUpgrades {
    pub choices: Vec<UpgradeCard>,
}

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        let mut upgrade_pool = UpgradePool::default();
        upgrade_pool.initialize();
        app.insert_resource(upgrade_pool);
    }
}