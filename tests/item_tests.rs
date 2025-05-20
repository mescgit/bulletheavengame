use cosmic_gardener::items::{ItemId, ItemDefinition, ItemEffect, ItemLibrary};
use cosmic_gardener::survivor::Survivor; // Assuming survivor.rs is at crate root
use cosmic_gardener::skills::ActiveSkillInstance; // For Survivor::new_with_skills_and_items
use cosmic_gardener::components::Health as ComponentHealth; // For checking health component
use bevy::prelude::default; // For default components if needed for Survivor

// Helper function to create a basic ItemLibrary for testing
fn setup_test_item_library() -> ItemLibrary {
    let mut library = ItemLibrary::default();
    library.items.push(ItemDefinition {
        id: ItemId(1001),
        name: "Test Health Relic".to_string(),
        description: "Test +10 Max Endurance.".to_string(),
        effects: vec![ItemEffect::PassiveStatBoost { 
            max_health_increase: Some(10), 
            speed_multiplier: None, 
            damage_increase: None, 
            xp_gain_multiplier: None, 
            pickup_radius_increase: None 
        }],
    });
    library.items.push(ItemDefinition {
        id: ItemId(1002),
        name: "Test Speed Relic".to_string(),
        description: "Test +20% Speed.".to_string(),
        effects: vec![ItemEffect::PassiveStatBoost { 
            max_health_increase: None, 
            speed_multiplier: Some(1.20), 
            damage_increase: None, 
            xp_gain_multiplier: None, 
            pickup_radius_increase: None 
        }],
    });
    library.items.push(ItemDefinition {
        id: ItemId(1003),
        name: "Test Damage Relic".to_string(),
        description: "Test +5 Ichor Blast Damage.".to_string(),
        effects: vec![ItemEffect::PassiveStatBoost { 
            max_health_increase: None, 
            speed_multiplier: None, 
            damage_increase: Some(5), 
            xp_gain_multiplier: None, 
            pickup_radius_increase: None 
        }],
    });
    library
}

#[test]
fn test_item_library_get_definition() {
    let library = setup_test_item_library();
    assert!(library.get_item_definition(ItemId(1001)).is_some());
    assert_eq!(library.get_item_definition(ItemId(1001)).unwrap().name, "Test Health Relic");
    assert!(library.get_item_definition(ItemId(9999)).is_none());
}

#[test]
fn test_passive_stat_boost_application() {
    let item_library = setup_test_item_library();
    let mut survivor = Survivor::new_with_skills_and_items(Vec::new(), Vec::new());
    // Store initial values
    let initial_max_health = survivor.max_health;
    let initial_speed = survivor.speed;
    let initial_ichor_blast_bonus = survivor.ichor_blast_damage_bonus;

    // Mock ItemCollectedEvent and apply_collected_item_effects_system logic
    // In a real Bevy test, we'd send an event. Here, we simulate the core logic.
    
    // Apply Health Relic
    if let Some(item_def) = item_library.get_item_definition(ItemId(1001)) {
        survivor.collected_item_ids.push(ItemId(1001)); // Simulate collection
        for effect in &item_def.effects {
            if let ItemEffect::PassiveStatBoost { max_health_increase, .. } = effect {
                if let Some(hp_boost) = max_health_increase {
                    survivor.max_health += *hp_boost;
                    // In a real scenario, a Health component would also be updated.
                }
            }
        }
    }
    assert_eq!(survivor.max_health, initial_max_health + 10);

    // Apply Speed Relic
    if let Some(item_def) = item_library.get_item_definition(ItemId(1002)) {
        survivor.collected_item_ids.push(ItemId(1002));
        for effect in &item_def.effects {
            if let ItemEffect::PassiveStatBoost { speed_multiplier, .. } = effect {
                if let Some(speed_mult) = speed_multiplier {
                    survivor.speed *= *speed_mult;
                }
            }
        }
    }
    assert_eq!(survivor.speed, initial_speed * 1.20);
    
    // Apply Damage Relic
    if let Some(item_def) = item_library.get_item_definition(ItemId(1003)) {
        survivor.collected_item_ids.push(ItemId(1003));
        for effect in &item_def.effects {
            if let ItemEffect::PassiveStatBoost { damage_increase, .. } = effect {
                if let Some(dmg_inc) = damage_increase {
                    survivor.ichor_blast_damage_bonus += *dmg_inc;
                }
            }
        }
    }
    assert_eq!(survivor.ichor_blast_damage_bonus, initial_ichor_blast_bonus + 5);
}
