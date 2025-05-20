use cosmic_gardener::skills::{ActiveSkillInstance, SkillId}; // Assuming 'cosmic_gardener' is the crate name
use std::time::Duration;

#[test]
fn test_active_skill_instance_new() {
    let skill_instance = ActiveSkillInstance::new(SkillId(1), 2);
    assert_eq!(skill_instance.definition_id, SkillId(1));
    assert_eq!(skill_instance.current_cooldown, Duration::ZERO);
    assert_eq!(skill_instance.current_level, 1);
    assert_eq!(skill_instance.flat_damage_bonus, 0);
    assert_eq!(skill_instance.cooldown_multiplier, 1.0);
    assert_eq!(skill_instance.aoe_radius_multiplier, 1.0);
    assert_eq!(skill_instance.equipped_glyphs.len(), 2);
    assert!(skill_instance.equipped_glyphs.iter().all(|g| g.is_none()));
}

#[test]
fn test_active_skill_instance_trigger() {
    let mut skill_instance = ActiveSkillInstance::new(SkillId(1), 0);
    let base_cooldown = Duration::from_secs_f32(2.0);
    
    assert!(skill_instance.is_ready());
    skill_instance.trigger(base_cooldown);
    assert!(!skill_instance.is_ready());
    assert_eq!(skill_instance.current_cooldown, base_cooldown);

    // Test with cooldown multiplier
    skill_instance.current_cooldown = Duration::ZERO; // Reset cooldown
    skill_instance.cooldown_multiplier = 0.5;
    skill_instance.trigger(base_cooldown);
    assert_eq!(skill_instance.current_cooldown, Duration::from_secs_f32(1.0));
}

#[test]
fn test_active_skill_instance_tick_cooldown() {
    let mut skill_instance = ActiveSkillInstance::new(SkillId(1), 0);
    skill_instance.current_cooldown = Duration::from_secs(5);
    
    skill_instance.tick_cooldown(Duration::from_secs(1));
    assert_eq!(skill_instance.current_cooldown, Duration::from_secs(4));

    skill_instance.tick_cooldown(Duration::from_secs(5)); // Tick past zero
    assert_eq!(skill_instance.current_cooldown, Duration::ZERO);
}
