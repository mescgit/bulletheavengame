use bevy::prelude::*;
use rand::seq::SliceRandom;
use crate::skills::SkillId;

#[derive(Debug, Clone, PartialEq)]
pub enum UpgradeType {
    SurvivorSpeed(u32), MaxEndurance(i32), IchorBlastIntensity(i32), IchorBlastSpeed(u32), IchorBlastVelocity(u32), IchorBlastPiercing(u32),
    EchoesGainMultiplier(u32), SoulAttractionRadius(u32), AdditionalIchorBlasts(u32), InscribeCircleOfWarding,
    IncreaseCircleRadius(u32), IncreaseCircleDamage(i32), DecreaseCircleTickRate(u32), EnduranceRegeneration(f32),
    ManifestSwarmOfNightmares, IncreaseNightmareCount(u32), IncreaseNightmareDamage(i32), IncreaseNightmareRadius(f32), IncreaseNightmareRotationSpeed(f32),
    IncreaseSkillDamage { slot_index: usize, amount: i32 }, GrantRandomRelic, GrantSkill(SkillId),
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
            // Survivor Stats
            UpgradeCard {id: UpgradeId(0), name: "Borrowed Swiftness".to_string(), description: "Your limbs move with uncanny swiftness borrowed from beyond. +10% speed.".to_string(), upgrade_type: UpgradeType::SurvivorSpeed(10),},
            UpgradeCard {id: UpgradeId(1), name: "Flesh-Bound Pact".to_string(), description: "A pact seals your flesh against oblivion. +20 Max Endurance.".to_string(), upgrade_type: UpgradeType::MaxEndurance(20),},
            UpgradeCard {id: UpgradeId(5), name: "Otherworldly Agility".to_string(), description: "You glide like a creature not of this realm. +15% speed.".to_string(), upgrade_type: UpgradeType::SurvivorSpeed(15),},
            UpgradeCard {id: UpgradeId(6), name: "Resilient Corpus".to_string(), description: "Your form knits itself against harsher realities. +30 Max Endurance.".to_string(), upgrade_type: UpgradeType::MaxEndurance(30),},
            UpgradeCard {id: UpgradeId(300), name: "Unnatural Vigor".to_string(), description: "Reality warps to mend your wounds. Regenerate 0.5 Endurance/sec.".to_string(), upgrade_type: UpgradeType::EnduranceRegeneration(0.5),},
            UpgradeCard {id: UpgradeId(301), name: "Bound by Ichor".to_string(), description: "Strange energies sustain your form. Regenerate 1.0 Endurance/sec.".to_string(), upgrade_type: UpgradeType::EnduranceRegeneration(1.0),},

            // Ichor Blast (Main Attack)
            UpgradeCard {id: UpgradeId(2), name: "Maddening Focus".to_string(), description: "Your ichor blasts strike with greater force. +5 Ichor Blast damage.".to_string(), upgrade_type: UpgradeType::IchorBlastIntensity(5),},
            UpgradeCard {id: UpgradeId(3), name: "Rapid Sanity Strain".to_string(), description: "Your mind strains faster, casting ichor blasts more quickly. +15% cast speed.".to_string(), upgrade_type: UpgradeType::IchorBlastSpeed(15),},
            UpgradeCard {id: UpgradeId(4), name: "Swift Ichor".to_string(), description: "Your Ichor Blasts travel faster. +20% velocity.".to_string(), upgrade_type: UpgradeType::IchorBlastVelocity(20),},
            UpgradeCard {id: UpgradeId(7), name: "Piercing Ichor".to_string(), description: "Your ichor blasts carry deeper malevolence. +8 Ichor Blast damage.".to_string(), upgrade_type: UpgradeType::IchorBlastIntensity(8),},
            UpgradeCard {id: UpgradeId(8), name: "Hyper Sanity Strain".to_string(), description: "Your mind strains with startling alacrity, casting ichor blasts faster. +20% cast speed.".to_string(), upgrade_type: UpgradeType::IchorBlastSpeed(20),},
            UpgradeCard {id: UpgradeId(9), name: "Unraveling Ichor".to_string(), description: "Your Ichor Blasts tear through more horrors. Pierce +1 horror.".to_string(), upgrade_type: UpgradeType::IchorBlastPiercing(1),},
            UpgradeCard {id: UpgradeId(12), name: "Persistent Ichor".to_string(), description: "Your Ichor Blasts linger longer in reality. Pierce +2 horrors.".to_string(), upgrade_type: UpgradeType::IchorBlastPiercing(2),},
            UpgradeCard {id: UpgradeId(200), name: "Fractured Sanity".to_string(), description: "Your mind splinters, projecting an additional ichor blast. +1 Ichor Blast.".to_string(), upgrade_type: UpgradeType::AdditionalIchorBlasts(1),},
            UpgradeCard {id: UpgradeId(201), name: "Ichor Barrage".to_string(), description: "Your consciousness erupts, projecting two additional ichor blasts. +2 Ichor Blasts.".to_string(), upgrade_type: UpgradeType::AdditionalIchorBlasts(2),},

            // Echoes (XP) & Pickups
            UpgradeCard {id: UpgradeId(10), name: "Glimpse Beyond The Veil".to_string(), description: "Glimpses of the abyss accelerate your horrific understanding. +20% Echoes gain.".to_string(), upgrade_type: UpgradeType::EchoesGainMultiplier(20),},
            UpgradeCard {id: UpgradeId(11), name: "Soul Grasp".to_string(), description: "The echoes of fallen horrors are drawn to you. +25% Echoing Soul attraction radius.".to_string(), upgrade_type: UpgradeType::SoulAttractionRadius(25),},
            UpgradeCard {id: UpgradeId(13), name: "Abyssal Understanding".to_string(), description: "You perceive deeper truths, hastening your evolution. +30% Echoes gain.".to_string(), upgrade_type: UpgradeType::EchoesGainMultiplier(30),},
            
            // Circle of Warding (Aura Weapon)
            UpgradeCard {id: UpgradeId(100), name: "Inscribe Circle of Warding".to_string(), description: "Manifest an aura of protective, damaging glyphs.".to_string(), upgrade_type: UpgradeType::InscribeCircleOfWarding,},
            UpgradeCard {id: UpgradeId(101), name: "Echoing Wards".to_string(), description: "Your protective circle extends further. +20% circle radius.".to_string(), upgrade_type: UpgradeType::IncreaseCircleRadius(20),},
            UpgradeCard {id: UpgradeId(102), name: "Maddening Wards".to_string(), description: "Your circle inflicts greater mental anguish. +2 circle damage.".to_string(), upgrade_type: UpgradeType::IncreaseCircleDamage(2),},
            UpgradeCard {id: UpgradeId(103), name: "Frenzied Wards".to_string(), description: "Your circle pulses with greater frequency. Circle damages 15% faster.".to_string(), upgrade_type: UpgradeType::DecreaseCircleTickRate(15),},

            // Swarm of Nightmares (Orbiter Weapon)
            UpgradeCard {id: UpgradeId(400), name: "Manifest Swarm of Nightmares".to_string(), description: "Conjure 2 nightmare larva that orbit and attack foes.".to_string(), upgrade_type: UpgradeType::ManifestSwarmOfNightmares,},
            UpgradeCard {id: UpgradeId(401), name: "Grow the Nightmare Swarm".to_string(), description: "Add another Nightmare Larva to your psychic defenses. +1 nightmare.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareCount(1),},
            UpgradeCard {id: UpgradeId(402), name: "Venomous Nightmares".to_string(), description: "Your Nightmare Larva inflict deeper wounds. +3 nightmare damage.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareDamage(3),},
            UpgradeCard {id: UpgradeId(403), name: "Extended Nightmare Patrol".to_string(), description: "Your Nightmare Larva patrol a wider area. +15 orbit radius.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareRadius(15.0),},
            UpgradeCard {id: UpgradeId(404), name: "Swifter Nightmares".to_string(), description: "Your Nightmare Larva move with increased speed. +0.5 rad/s orbit speed.".to_string(), upgrade_type: UpgradeType::IncreaseNightmareRotationSpeed(0.5),},
            
            // Skill Specific Upgrades
            UpgradeCard {id: UpgradeId(500), name: "Empower Eldritch Bolt".to_string(), description: "Increase Eldritch Bolt damage by 10.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 0, amount: 10 },},
            UpgradeCard {id: UpgradeId(501), name: "Intensify Mind Shatter".to_string(), description: "Mind Shatter fragments each deal +3 damage.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 1, amount: 3 },}, // Changed
            UpgradeCard {id: UpgradeId(502), name: "Sharpen Void Lance".to_string(), description: "Increase Void Lance damage by 20.".to_string(), upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 2, amount: 20 },},
            
            // General/Utility
            UpgradeCard {id: UpgradeId(600), name: "Mysterious Relic".to_string(), description: "The abyss grants you a random relic.".to_string(), upgrade_type: UpgradeType::GrantRandomRelic,},

            // Grant Skills
            UpgradeCard {id: UpgradeId(700), name: "Learn: Mind Shatter".to_string(), description: "Unlock the Mind Shatter psychic burst skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(2)),},
            UpgradeCard {id: UpgradeId(701), name: "Learn: Void Lance".to_string(), description: "Unlock the Void Lance piercing projectile skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(3)),},
            UpgradeCard {id: UpgradeId(702), name: "Learn: Fleeting Agility".to_string(), description: "Unlock the Fleeting Agility self-buff skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(4)),},
            UpgradeCard {id: UpgradeId(703), name: "Learn: Glacial Nova".to_string(), description: "Unlock the Glacial Nova chilling skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(5)),},
            UpgradeCard {id: UpgradeId(704), name: "Learn: Psychic Sentry".to_string(), description: "Unlock the Psychic Sentry summon skill.".to_string(), upgrade_type: UpgradeType::GrantSkill(SkillId(6)),},

            // Skill Meta Upgrades
            UpgradeCard {id: UpgradeId(800), name: "Echoing Bolt".to_string(), description: "Eldritch Bolt recharges 15% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 0, percent_reduction: 0.15 },},
            UpgradeCard {id: UpgradeId(801), name: "Focused Mind Shatter".to_string(), description: "Mind Shatter recharges 15% faster.".to_string(), upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 1, percent_reduction: 0.15 },}, // Changed
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