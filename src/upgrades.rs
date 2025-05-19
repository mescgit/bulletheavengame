use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::skills::SkillId;

#[derive(Debug, Clone, PartialEq)]
pub enum UpgradeType {
    PlayerSpeed(u32), MaxHealth(i32), FocusIntensity(i32), FocusSpeed(u32), FragmentVelocity(u32), FragmentPiercing(u32),
    XPGainMultiplier(u32), PickupRadiusIncrease(u32), AdditionalMindFragments(u32), UnleashWardingWhispers,
    IncreaseAoeAuraRadius(u32), IncreaseAoeAuraDamage(i32), DecreaseAoeAuraTickRate(u32), HealthRegeneration(f32),
    ManifestMindLarvae, IncreaseOrbiterCount(u32), IncreaseOrbiterDamage(i32), IncreaseOrbiterRadius(f32), IncreaseOrbiterRotationSpeed(f32),
    IncreaseSkillDamage { slot_index: usize, amount: i32 }, GrantRandomItem, GrantSkill(SkillId),
    ReduceSkillCooldown { slot_index: usize, percent_reduction: f32 }, IncreaseSkillAoERadius { slot_index: usize, percent_increase: f32 },
}

#[derive(Debug, Clone)]
pub struct UpgradeCard { pub id: UpgradeId, pub name: String, pub description: String, pub upgrade_type: UpgradeType, }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpgradeId(pub u32);

#[derive(Resource, Default)]
pub struct UpgradePool { pub available_upgrades: Vec<UpgradeCard>, }

impl UpgradePool {
    pub fn initialize(&mut self) {
        self.available_upgrades = vec![
            UpgradeCard {id: UpgradeId(0), name: "Preternatural Speed".to_string(), description: "Your limbs move with uncanny swiftness. +10% speed.".to_string(), upgrade_type: UpgradeType::PlayerSpeed(10),},
            UpgradeCard {id: UpgradeId(1), name: "Enduring Form".to_string(), description: "Your physical vessel resists oblivion. +20 Max Health.".to_string(), upgrade_type: UpgradeType::MaxHealth(20),},
            UpgradeCard {id: UpgradeId(5), name: "Otherworldly Agility".to_string(), description: "You glide like a creature not of this realm. +15% speed.".to_string(), upgrade_type: UpgradeType::PlayerSpeed(15),},
            UpgradeCard {id: UpgradeId(6), name: "Resilient Corpus".to_string(), description: "Your form knits itself against harsher realities. +30 Max Health.".to_string(), upgrade_type: UpgradeType::MaxHealth(30),},
            UpgradeCard {id: UpgradeId(300), name: "Unnatural Vigor".to_string(), description: "Reality warps to mend your wounds. Regenerate 0.5 HP/sec.".to_string(), upgrade_type: UpgradeType::HealthRegeneration(0.5),},
            UpgradeCard {id: UpgradeId(301), name: "Bound by Ichor".to_string(), description: "Strange energies sustain your form. Regenerate 1.0 HP/sec.".to_string(), upgrade_type: UpgradeType::HealthRegeneration(1.0),},
            UpgradeCard {id: UpgradeId(2), name: "Focused Will".to_string(), description: "Your projected thoughts strike with greater force. +5 Mind Fragment damage.".to_string(), upgrade_type: UpgradeType::FocusIntensity(5),},
            UpgradeCard {id: UpgradeId(3), name: "Rapid Cognition".to_string(), description: "Your mind projects fragments faster. +15% projection speed.".to_string(), upgrade_type: UpgradeType::FocusSpeed(15),},
            UpgradeCard {id: UpgradeId(4), name: "Swift Thoughts".to_string(), description: "Your Mind Fragments travel faster. +20% velocity.".to_string(), upgrade_type: UpgradeType::FragmentVelocity(20),},
            UpgradeCard {id: UpgradeId(7), name: "Piercing Thoughts".to_string(), description: "Your projected thoughts carry deeper malevolence. +8 Mind Fragment damage.".to_string(), upgrade_type: UpgradeType::FocusIntensity(8),},
            UpgradeCard {id: UpgradeId(8), name: "Hyper Cognition".to_string(), description: "Your mind projects fragments with startling alacrity. +20% projection speed.".to_string(), upgrade_type: UpgradeType::FocusSpeed(20),},
            UpgradeCard {id: UpgradeId(9), name: "Unraveling Thoughts".to_string(), description: "Your Mind Fragments tear through more foes. Pierce +1 enemy.".to_string(), upgrade_type: UpgradeType::FragmentPiercing(1),},
            UpgradeCard {id: UpgradeId(12), name: "Persistent Thoughts".to_string(), description: "Your Mind Fragments linger longer in reality. Pierce +2 enemies.".to_string(), upgrade_type: UpgradeType::FragmentPiercing(2),},
            UpgradeCard {id: UpgradeId(200), name: "Fractured Thoughts".to_string(), description: "Your mind splinters, projecting an additional fragment. +1 Mind Fragment.".to_string(), upgrade_type: UpgradeType::AdditionalMindFragments(1),},
            UpgradeCard {id: UpgradeId(201), name: "Thought Barrage".to_string(), description: "Your consciousness erupts, projecting two additional fragments. +2 Mind Fragments.".to_string(), upgrade_type: UpgradeType::AdditionalMindFragments(2),},
            UpgradeCard {id: UpgradeId(10), name: "Forbidden Knowledge".to_string(), description: "Glimpses of the abyss accelerate your growth. +20% XP gain.".to_string(), upgrade_type: UpgradeType::XPGainMultiplier(20),},
            UpgradeCard {id: UpgradeId(11), name: "Psychic Grasp".to_string(), description: "The echoes of fallen minds are drawn to you. +25% XP orb attraction radius.".to_string(), upgrade_type: UpgradeType::PickupRadiusIncrease(25),},
            UpgradeCard {id: UpgradeId(13), name: "Cosmic Understanding".to_string(), description: "You perceive deeper truths, hastening your evolution. +30% XP gain.".to_string(), upgrade_type: UpgradeType::XPGainMultiplier(30),},
            UpgradeCard {id: UpgradeId(100), name: "Unleash Warding Whispers".to_string(), description: "Manifest an aura of protective, damaging whispers.".to_string(), upgrade_type: UpgradeType::UnleashWardingWhispers,},
            UpgradeCard {id: UpgradeId(101), name: "Echoing Whispers".to_string(), description: "Your protective whispers extend further. +20% aura radius.".to_string(), upgrade_type: UpgradeType::IncreaseAoeAuraRadius(20),},
            UpgradeCard {id: UpgradeId(102), name: "Maddening Whispers".to_string(), description: "Your whispers inflict greater mental anguish. +2 aura damage.".to_string(), upgrade_type: UpgradeType::IncreaseAoeAuraDamage(2),},
            UpgradeCard {id: UpgradeId(103), name: "Frenzied Whispers".to_string(), description: "Your whispers pulse with greater frequency. Aura damages 15% faster.".to_string(), upgrade_type: UpgradeType::DecreaseAoeAuraTickRate(15),},
            UpgradeCard {id: UpgradeId(400), name: "Summon Mind Larva".to_string(), description: "Conjure 2 psychic larva that orbit and attack foes.".to_string(), upgrade_type: UpgradeType::ManifestMindLarvae,},
            UpgradeCard {id: UpgradeId(401), name: "Grow the Swarm".to_string(), description: "Add another Mind Larva to your psychic defenses. +1 orbiter.".to_string(), upgrade_type: UpgradeType::IncreaseOrbiterCount(1),},
            UpgradeCard {id: UpgradeId(402), name: "Venomous Larva".to_string(), description: "Your Mind Larva inflict deeper wounds. +3 orbiter damage.".to_string(), upgrade_type: UpgradeType::IncreaseOrbiterDamage(3),},
            UpgradeCard {id: UpgradeId(403), name: "Extended Patrol".to_string(), description: "Your Mind Larva patrol a wider area. +15 orbit radius.".to_string(), upgrade_type: UpgradeType::IncreaseOrbiterRadius(15.0),},
            UpgradeCard {id: UpgradeId(404), name: "Swifter Larva".to_string(), description: "Your Mind Larva move with increased speed. +0.5 rad/s orbit speed.".to_string(), upgrade_type: UpgradeType::IncreaseOrbiterRotationSpeed(0.5),},
            UpgradeCard {id: UpgradeId(500), name: "Empower Eldritch Bolt".to_string(), description: "Increase Eldritch Bolt damage by 10.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 0, amount: 10 },},
            UpgradeCard {id: UpgradeId(501), name: "Intensify Mind Shatter".to_string(), description: "Increase Mind Shatter damage by 15.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 1, amount: 15 },},
            UpgradeCard {id: UpgradeId(502), name: "Sharpen Void Lance".to_string(), description: "Increase Void Lance damage by 20.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 2, amount: 20 },},
            UpgradeCard {id: UpgradeId(600), name: "Mysterious Boon".to_string(), description: "The cosmos grants you a random artifact.".to_string(), upgrade_type: UpgradeType::GrantRandomItem,},
            UpgradeCard {id: UpgradeId(700), name: "Learn: Mind Shatter".to_string(), description: "Unlock the Mind Shatter psychic burst skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(2)),},
            UpgradeCard {id: UpgradeId(701), name: "Learn: Void Lance".to_string(), description: "Unlock the Void Lance piercing projectile skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(3)),},
            UpgradeCard {id: UpgradeId(702), name: "Learn: Fleeting Agility".to_string(), description: "Unlock the Fleeting Agility self-buff skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(4)),},
            // Removed Abyssal Snare Learn Card, SkillId(5) is now Glacial Nova
            UpgradeCard {id: UpgradeId(703), name: "Learn: Glacial Nova".to_string(), description: "Unlock the Glacial Nova chilling skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(5)),},
            UpgradeCard {id: UpgradeId(704), name: "Learn: Psychic Sentry".to_string(), description: "Unlock the Psychic Sentry summon skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(6)),},
            UpgradeCard {id: UpgradeId(800), name: "Echoing Bolt".to_string(), description: "Eldritch Bolt recharges 15% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 0, percent_reduction: 0.15 },},
            UpgradeCard {id: UpgradeId(801), name: "Rippling Shatter".to_string(), description: "Mind Shatter's area of effect expands by 20%.".to_string(), upgrade_type: UpgradeType::IncreaseSkillAoERadius { slot_index: 1, percent_increase: 0.20 },},
            UpgradeCard {id: UpgradeId(802), name: "Accelerated Void".to_string(), description: "Void Lance recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 2, percent_reduction: 0.10 },},
            UpgradeCard {id: UpgradeId(803), name: "Heightened Reflexes".to_string(), description: "Fleeting Agility recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 3, percent_reduction: 0.10 },},
            UpgradeCard {id: UpgradeId(804), name: "Cryo-Resonance".to_string(), description: "Glacial Nova recharges 10% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 4, percent_reduction: 0.10 },}, // Index 4 if Glacial Nova is 5th skill
            UpgradeCard {id: UpgradeId(805), name: "Expanded Chill".to_string(), description: "Glacial Nova's area of effect expands by 15%.".to_string(), upgrade_type: UpgradeType::IncreaseSkillAoERadius { slot_index: 4, percent_increase: 0.15 },},
        ];
    }
    pub fn get_random_upgrades(&self, count: usize) -> Vec<UpgradeCard> { let mut rng = rand::thread_rng(); self.available_upgrades.choose_multiple(&mut rng, count).cloned().collect() }
}

#[derive(Component, Debug, Clone)] pub struct OfferedUpgrades { pub choices: Vec<UpgradeCard>, }
pub struct UpgradePlugin;
impl Plugin for UpgradePlugin { fn build(&self, app: &mut App) { let mut upgrade_pool = UpgradePool::default(); upgrade_pool.initialize(); app.insert_resource(upgrade_pool); } }