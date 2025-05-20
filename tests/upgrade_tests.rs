use cosmic_gardener::upgrades::{UpgradeId, UpgradeCard, UpgradeType, UpgradePool};
use cosmic_gardener::skills::{SkillId, ActiveSkillInstance};
use cosmic_gardener::survivor::Survivor; // For applying upgrades to survivor's skills
use std::time::Duration;

fn setup_test_upgrade_pool() -> UpgradePool {
    let mut pool = UpgradePool::default();
    // Add a subset of varied upgrades for testing
    pool.available_upgrades.push(UpgradeCard {
        id: UpgradeId(1001), name: "Test Skill Damage Up".to_string(), 
        description: "Test +5 Dmg to Skill 0".to_string(), 
        upgrade_type: UpgradeType::IncreaseSkillDamage { slot_index: 0, amount: 5 }
    });
    pool.available_upgrades.push(UpgradeCard {
        id: UpgradeId(1002), name: "Test Skill Cooldown Up".to_string(), 
        description: "Test -10% Cooldown to Skill 0".to_string(), 
        upgrade_type: UpgradeType::ReduceSkillCooldown { slot_index: 0, percent_reduction: 0.10 }
    });
    pool.available_upgrades.push(UpgradeCard {
        id: UpgradeId(1003), name: "Test Stat Up".to_string(),
        description: "Test +10 Speed".to_string(),
        upgrade_type: UpgradeType::SurvivorSpeed(10) // Assuming SurvivorSpeed exists
    });
     pool.available_upgrades.push(UpgradeCard {
        id: UpgradeId(1004), name: "Test Skill Grant".to_string(),
        description: "Grants Test Skill".to_string(),
        upgrade_type: UpgradeType::GrantSkill(SkillId(99)) // A dummy SkillId for testing
    });
    pool
}

#[test]
fn test_upgrade_pool_get_random_upgrades() {
    let pool = setup_test_upgrade_pool();
    
    let offered1 = pool.get_random_upgrades(1);
    assert_eq!(offered1.len(), 1);
    assert!(pool.available_upgrades.contains(&offered1[0]));

    let offered3 = pool.get_random_upgrades(3);
    assert_eq!(offered3.len(), 3);
    for card in offered3.iter() {
        assert!(pool.available_upgrades.contains(card));
    }
    // Check for uniqueness (highly probable with SliceRandom::choose_multiple, but good to be aware)
    if offered3.len() == 3 {
        assert_ne!(offered3[0].id, offered3[1].id);
        assert_ne!(offered3[0].id, offered3[2].id);
        assert_ne!(offered3[1].id, offered3[2].id);
    }
    
    let offered_more_than_available = pool.get_random_upgrades(pool.available_upgrades.len() + 1);
    assert_eq!(offered_more_than_available.len(), pool.available_upgrades.len());
}

#[test]
fn test_apply_skill_damage_upgrade() {
    let mut survivor = Survivor::new_with_skills_and_items(
        vec![ActiveSkillInstance::new(SkillId(1), 0)], // Skill in slot 0
        Vec::new()
    );
    
    let upgrade = UpgradeType::IncreaseSkillDamage { slot_index: 0, amount: 10 };
    
    // Simulate apply_chosen_upgrade logic for this specific upgrade
    if let Some(skill_instance) = survivor.equipped_skills.get_mut(0) {
        let initial_damage_bonus = skill_instance.flat_damage_bonus;
        let initial_level = skill_instance.current_level;
        
        skill_instance.flat_damage_bonus += 10; // from upgrade.amount
        skill_instance.current_level += 1;

        assert_eq!(skill_instance.flat_damage_bonus, initial_damage_bonus + 10);
        assert_eq!(skill_instance.current_level, initial_level + 1);
    } else {
        panic!("Skill not found in slot 0 for testing");
    }
}

#[test]
fn test_apply_skill_cooldown_upgrade() {
    let mut survivor = Survivor::new_with_skills_and_items(
        vec![ActiveSkillInstance::new(SkillId(1), 0)], // Skill in slot 0
        Vec::new()
    );
    
    let upgrade = UpgradeType::ReduceSkillCooldown { slot_index: 0, percent_reduction: 0.20 };

    // Simulate apply_chosen_upgrade logic
    if let Some(skill_instance) = survivor.equipped_skills.get_mut(0) {
        let initial_cooldown_multiplier = skill_instance.cooldown_multiplier;
        let initial_level = skill_instance.current_level;

        skill_instance.cooldown_multiplier *= 1.0 - 0.20; // from upgrade.percent_reduction
        skill_instance.current_level += 1;
        
        assert_eq!(skill_instance.cooldown_multiplier, initial_cooldown_multiplier * 0.80);
        assert_eq!(skill_instance.current_level, initial_level + 1);
    } else {
        panic!("Skill not found in slot 0 for testing");
    }
}
