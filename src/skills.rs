use bevy::prelude::*;
use std::time::Duration;
use crate::{
    survivor::{Survivor, SURVIVOR_SIZE}, // Changed
    game::AppState,
    components::{Velocity, Damage, Lifetime, Health},
    horror::Horror, // Changed
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
    glyphs::{GlyphId, GlyphLibrary, GlyphEffectType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub struct SkillId(pub u32);

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SkillEffectType {
    Projectile {
        base_damage: i32,
        speed: f32,
        size: Vec2,
        color: Color,
        lifetime_secs: f32,
        piercing: u32,
    },
    AreaOfEffect {
        base_damage_per_tick: i32,
        base_radius: f32,
        tick_interval_secs: f32,
        duration_secs: f32,
        color: Color,
    },
    SurvivorBuff { // Changed
        speed_multiplier_bonus: f32,
        fire_rate_multiplier_bonus: f32,
        duration_secs: f32,
    },
    SummonSentry {
        sentry_damage_per_tick: i32,
        sentry_radius: f32,
        sentry_tick_interval_secs: f32,
        sentry_duration_secs: f32,
        sentry_color: Color,
    },
    FreezingNova {
        damage: i32,
        radius: f32,
        nova_duration_secs: f32,
        slow_multiplier: f32,
        slow_duration_secs: f32,
        color: Color,
    },
}

#[derive(Debug, Clone, Reflect)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub base_cooldown: Duration,
    pub effect: SkillEffectType,
    pub base_glyph_slots: u8,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct ActiveSkillInstance {
    pub definition_id: SkillId,
    pub current_cooldown: Duration,
    pub current_level: u32,
    pub flat_damage_bonus: i32,
    pub cooldown_multiplier: f32,
    pub aoe_radius_multiplier: f32,
    pub equipped_glyphs: Vec<Option<GlyphId>>,
}

impl ActiveSkillInstance {
    pub fn new(definition_id: SkillId, base_glyph_slots: u8) -> Self { Self { definition_id, current_cooldown: Duration::ZERO, current_level: 1, flat_damage_bonus: 0, cooldown_multiplier: 1.0, aoe_radius_multiplier: 1.0, equipped_glyphs: vec![None; base_glyph_slots as usize], } }
    pub fn tick_cooldown(&mut self, delta: Duration) { if self.current_cooldown > Duration::ZERO { self.current_cooldown = self.current_cooldown.saturating_sub(delta); } }
    pub fn is_ready(&self) -> bool { self.current_cooldown == Duration::ZERO }
    pub fn trigger(&mut self, base_cooldown: Duration) { let modified_cooldown_secs = base_cooldown.as_secs_f32() * self.cooldown_multiplier; self.current_cooldown = Duration::from_secs_f32(modified_cooldown_secs.max(0.1)); }
}

#[derive(Component)]
pub struct SkillProjectile {
    pub skill_id: SkillId,
    pub piercing_left: u32,
    pub bounces_left: u32,
    pub already_hit_by_this_projectile: Vec<Entity>, // Tracks entities hit by this specific projectile instance
}

#[derive(Component)] pub struct ActiveSkillAoEEffect { pub skill_id: SkillId, pub actual_damage_per_tick: i32, pub actual_radius_sq: f32, pub tick_timer: Timer, pub lifetime_timer: Timer, pub already_hit_this_tick: Vec<Entity>, }
#[derive(Component, Debug)] pub struct SurvivorBuffEffect { pub speed_multiplier_bonus: f32, pub fire_rate_multiplier_bonus: f32, pub duration_timer: Timer, }

#[derive(Component, Debug, Reflect, Default)] #[reflect(Component)]
pub struct FreezingNovaEffect { pub damage: i32, pub radius_sq: f32, pub lifetime_timer: Timer, pub slow_multiplier: f32, pub slow_duration_secs: f32, pub already_hit_entities: Vec<Entity>, }

#[derive(Resource, Default, Reflect)] #[reflect(Resource)]
pub struct SkillLibrary { pub skills: Vec<SkillDefinition>, }
impl SkillLibrary { pub fn get_skill_definition(&self, id: SkillId) -> Option<&SkillDefinition> { self.skills.iter().find(|def| def.id == id) } }

pub struct SkillsPlugin;
impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app .register_type::<SkillId>() .register_type::<SkillEffectType>() .register_type::<SkillDefinition>() .register_type::<ActiveSkillInstance>() .register_type::<SkillLibrary>()
            .register_type::<FreezingNovaEffect>()
            .init_resource::<SkillLibrary>()
            .add_systems(Startup, populate_skill_library)
            .add_systems(Update, ( active_skill_cooldown_recharge_system, survivor_skill_input_system, skill_projectile_lifetime_system, skill_projectile_collision_system, active_skill_aoe_system, survivor_buff_management_system, freezing_nova_effect_damage_system, // Renamed systems
            ).chain().run_if(in_state(AppState::InGame)) );
    }
}

fn populate_skill_library(mut library: ResMut<SkillLibrary>) {
    library.skills.push(SkillDefinition { id: SkillId(1), name: "Eldritch Bolt".to_string(), description: "Fires a bolt of arcane energy.".to_string(), base_cooldown: Duration::from_secs_f32(1.5), effect: SkillEffectType::Projectile { base_damage: 25, speed: 650.0, size: Vec2::new(12.0, 28.0), color: Color::rgb(0.6, 0.1, 0.9), lifetime_secs: 2.5, piercing: 0, }, base_glyph_slots: 2 });
    library.skills.push(SkillDefinition { id: SkillId(2), name: "Mind Shatter".to_string(), description: "Unleashes a short-range psychic burst in a wide arc.".to_string(), base_cooldown: Duration::from_secs(4), effect: SkillEffectType::AreaOfEffect { base_damage_per_tick: 35, base_radius: 175.0, tick_interval_secs: 0.1, duration_secs: 0.2, color: Color::rgba(0.8, 0.2, 1.0, 0.7), }, base_glyph_slots: 1 }); // Description updated
    library.skills.push(SkillDefinition { id: SkillId(3), name: "Void Lance".to_string(), description: "Projects a slow but potent lance of void energy that pierces foes.".to_string(), base_cooldown: Duration::from_secs_f32(2.5), effect: SkillEffectType::Projectile { base_damage: 40, speed: 400.0, size: Vec2::new(10.0, 40.0), color: Color::rgb(0.1, 0.0, 0.2), lifetime_secs: 3.0, piercing: 2, }, base_glyph_slots: 2 });
    library.skills.push(SkillDefinition { id: SkillId(4), name: "Fleeting Agility".to_string(), description: "Briefly enhance your speed and reflexes.".to_string(), base_cooldown: Duration::from_secs(20), effect: SkillEffectType::SurvivorBuff { speed_multiplier_bonus: 0.30, fire_rate_multiplier_bonus: 0.25, duration_secs: 5.0, }, base_glyph_slots: 0 }); // Changed
    library.skills.push(SkillDefinition { id: SkillId(5), name: "Glacial Nova".to_string(), description: "Emits a chilling nova, damaging and slowing nearby foes.".to_string(), base_cooldown: Duration::from_secs(10), effect: SkillEffectType::FreezingNova { damage: 20, radius: 200.0, nova_duration_secs: 0.5, slow_multiplier: 0.5, slow_duration_secs: 3.0, color: Color::rgba(0.5, 0.8, 1.0, 0.6), }, base_glyph_slots: 1, });
    library.skills.push(SkillDefinition { id: SkillId(6), name: "Psychic Sentry".to_string(), description: "Summons a stationary sentry that pulses with psychic energy.".to_string(), base_cooldown: Duration::from_secs(18), effect: SkillEffectType::SummonSentry { sentry_damage_per_tick: 15, sentry_radius: 100.0, sentry_tick_interval_secs: 0.75, sentry_duration_secs: 8.0, sentry_color: Color::rgba(0.2, 0.7, 0.9, 0.5), }, base_glyph_slots: 1 });
}

fn active_skill_cooldown_recharge_system(time: Res<Time>, mut player_query: Query<&mut Survivor>,) { if let Ok(mut player) = player_query.get_single_mut() { for skill_instance in player.equipped_skills.iter_mut() { skill_instance.tick_cooldown(time.delta()); } } }

fn survivor_skill_input_system( mut commands: Commands, asset_server: Res<AssetServer>, mouse_button_input: Res<ButtonInput<MouseButton>>, keyboard_input: Res<ButtonInput<KeyCode>>, mut player_query: Query<(Entity, &mut Survivor, &Transform)>, skill_library: Res<SkillLibrary>, glyph_library: Res<GlyphLibrary>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { // Renamed
    if let Ok((player_entity, mut player, player_transform)) = player_query.get_single_mut() {
        let mut skill_to_trigger_idx: Option<usize> = None;
        if mouse_button_input.just_pressed(MouseButton::Right) { skill_to_trigger_idx = Some(0); }
        else if keyboard_input.just_pressed(KeyCode::Digit1) { skill_to_trigger_idx = Some(0); }
        else if keyboard_input.just_pressed(KeyCode::Digit2) { skill_to_trigger_idx = Some(1); }
        else if keyboard_input.just_pressed(KeyCode::Digit3) { skill_to_trigger_idx = Some(2); }
        else if keyboard_input.just_pressed(KeyCode::KeyE) { skill_to_trigger_idx = Some(3); } 
        else if keyboard_input.just_pressed(KeyCode::KeyR) { skill_to_trigger_idx = Some(4); } 

        if let Some(idx) = skill_to_trigger_idx { if idx >= player.equipped_skills.len() { return; } let current_aim_direction = player.aim_direction; let skill_instance_snapshot = player.equipped_skills[idx].clone();
            if skill_instance_snapshot.is_ready() { if let Some(skill_def) = skill_library.get_skill_definition(skill_instance_snapshot.definition_id) {
                let mut effect_was_triggered = false; let mut projectile_damage = 0; let mut projectile_piercing = 0; let mut projectile_bounces = 0; let mut aoe_damage_per_tick = 0; let mut aoe_radius = 0.0; let mut sentry_damage_val = 0; let mut sentry_radius_val = 0.0; let mut nova_damage_val = 0; let mut nova_radius_val = 0.0;
                match &skill_def.effect { SkillEffectType::Projectile { base_damage, piercing: base_piercing, .. } => { projectile_damage = base_damage + skill_instance_snapshot.flat_damage_bonus; projectile_piercing = *base_piercing; } SkillEffectType::AreaOfEffect { base_damage_per_tick, base_radius, .. } => { aoe_damage_per_tick = base_damage_per_tick + skill_instance_snapshot.flat_damage_bonus; aoe_radius = base_radius * skill_instance_snapshot.aoe_radius_multiplier; }, SkillEffectType::SummonSentry { sentry_damage_per_tick: sdpt, sentry_radius: sr, ..} => { sentry_damage_val = sdpt + skill_instance_snapshot.flat_damage_bonus; sentry_radius_val = sr * skill_instance_snapshot.aoe_radius_multiplier; } SkillEffectType::FreezingNova { damage, radius, .. } => { nova_damage_val = damage + skill_instance_snapshot.flat_damage_bonus; nova_radius_val = radius * skill_instance_snapshot.aoe_radius_multiplier; } _ => {} }
                for glyph_opt in skill_instance_snapshot.equipped_glyphs.iter() { if let Some(glyph_id) = glyph_opt { if let Some(glyph_def) = glyph_library.get_glyph_definition(*glyph_id) { match &glyph_def.effect { GlyphEffectType::AddedChaosDamageToProjectile { damage_amount } => { if matches!(skill_def.effect, SkillEffectType::Projectile {..}) { projectile_damage += *damage_amount; } } GlyphEffectType::IncreasedAoEDamage { percent_increase } => { if matches!(skill_def.effect, SkillEffectType::AreaOfEffect {..}) { aoe_damage_per_tick = (aoe_damage_per_tick as f32 * (1.0 + percent_increase)).round() as i32; } if matches!(skill_def.effect, SkillEffectType::SummonSentry {..}) { sentry_damage_val = (sentry_damage_val as f32 * (1.0 + percent_increase)).round() as i32; } if matches!(skill_def.effect, SkillEffectType::FreezingNova {..}) { nova_damage_val = (nova_damage_val as f32 * (1.0 + percent_increase)).round() as i32; } } GlyphEffectType::ProjectileChain { bounces } => { if matches!(skill_def.effect, SkillEffectType::Projectile {..}) { projectile_bounces += bounces; } } } } } }
                match &skill_def.effect {
                    SkillEffectType::Projectile { speed, size, color, lifetime_secs, .. } => { if current_aim_direction != Vec2::ZERO { let projectile_spawn_position = player_transform.translation + current_aim_direction.extend(0.0) * (SURVIVOR_SIZE.y / 2.0 + size.y / 2.0); commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/eldritch_bolt_placeholder.png"), sprite: Sprite { custom_size: Some(*size), color: *color, ..default()}, transform: Transform::from_translation(projectile_spawn_position) .with_rotation(Quat::from_rotation_z(current_aim_direction.y.atan2(current_aim_direction.x))), ..default() }, SkillProjectile { skill_id: skill_def.id, piercing_left: projectile_piercing, bounces_left: projectile_bounces, already_hit_by_this_projectile: Vec::new()}, Velocity(current_aim_direction * *speed), Damage(projectile_damage), Lifetime { timer: Timer::from_seconds(*lifetime_secs, TimerMode::Once) }, Name::new(format!("SkillProjectile_{}", skill_def.name)), )); effect_was_triggered = true; } }
                    SkillEffectType::AreaOfEffect { base_damage_per_tick, .. } => { // Modified for Mind Shatter (SkillId(2))
                        if skill_def.id == SkillId(2) { // Mind Shatter
                            let num_projectiles = 5;
                            let spread_angle_rad = 60.0f32.to_radians(); // Total arc for projectiles
                            let angle_step = spread_angle_rad / (num_projectiles -1) as f32;
                            let base_angle = current_aim_direction.to_angle() - spread_angle_rad / 2.0;

                            for i in 0..num_projectiles {
                                let angle = base_angle + angle_step * i as f32;
                                let direction = Vec2::new(angle.cos(), angle.sin());
                                let projectile_spawn_position = player_transform.translation + direction.extend(0.0) * (SURVIVOR_SIZE.y / 2.0 + 10.0 / 2.0); // Using 10.0 as size.y for fragment
                                
                                let mind_shatter_damage = 15 + skill_instance_snapshot.flat_damage_bonus; // Using 15 as base, adjusted from AoE base_damage_per_tick
                                commands.spawn((
                                    SpriteBundle {
                                        texture: asset_server.load("sprites/mind_shatter_fragment_placeholder.png"),
                                        sprite: Sprite { custom_size: Some(Vec2::new(10.0, 10.0)), color: Color::rgba(0.8, 0.2, 1.0, 0.9), ..default()},
                                        transform: Transform::from_translation(projectile_spawn_position)
                                            .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
                                        ..default()
                                    },
                                    SkillProjectile {
                                        skill_id: skill_def.id,
                                        piercing_left: 0, // Or 1 if desired
                                        bounces_left: 0, // Mind Shatter fragments don't bounce by default
                                        already_hit_by_this_projectile: Vec::new(),
                                    },
                                    Velocity(direction * 400.0),
                                    Damage(mind_shatter_damage), // Use calculated damage
                                    Lifetime { timer: Timer::from_seconds(0.4, TimerMode::Once) }, // Short lifetime
                                    Name::new(format!("MindShatterFragment_{}", i)),
                                ));
                            }
                            effect_was_triggered = true;
                        } else { // Original AoE logic for other skills if any
                            let aoe_spawn_position = player_transform.translation; commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/mind_shatter_effect_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(aoe_radius * 2.0)), color: Color::rgba(0.8, 0.2, 1.0, 0.7), ..default()}, transform: Transform::from_translation(aoe_spawn_position.truncate().extend(0.2)), ..default() }, ActiveSkillAoEEffect { skill_id: skill_def.id, actual_damage_per_tick: aoe_damage_per_tick, actual_radius_sq: aoe_radius.powi(2), tick_timer: Timer::from_seconds(0.1/*tick_interval_secs*/, TimerMode::Repeating), lifetime_timer: Timer::from_seconds(0.2/*duration_secs*/, TimerMode::Once), already_hit_this_tick: Vec::new(), }, Name::new(format!("SkillAoE_{}", skill_def.name)), )); effect_was_triggered = true;
                        }
                    }
                    SkillEffectType::SurvivorBuff { speed_multiplier_bonus, fire_rate_multiplier_bonus, duration_secs } => { commands.entity(player_entity).insert(SurvivorBuffEffect { speed_multiplier_bonus: *speed_multiplier_bonus, fire_rate_multiplier_bonus: *fire_rate_multiplier_bonus, duration_timer: Timer::from_seconds(*duration_secs, TimerMode::Once), }); effect_was_triggered = true; }
                    SkillEffectType::SummonSentry { sentry_tick_interval_secs, sentry_duration_secs, sentry_color, .. } => { let sentry_spawn_position = player_transform.translation.truncate().extend(0.15); commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/psychic_sentry_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(sentry_radius_val * 0.5)), color: *sentry_color, ..default() }, transform: Transform::from_translation(sentry_spawn_position), ..default() }, ActiveSkillAoEEffect { skill_id: skill_def.id, actual_damage_per_tick: sentry_damage_val, actual_radius_sq: sentry_radius_val.powi(2), tick_timer: Timer::from_seconds(*sentry_tick_interval_secs, TimerMode::Repeating), lifetime_timer: Timer::from_seconds(*sentry_duration_secs, TimerMode::Once), already_hit_this_tick: Vec::new(), }, Name::new("PsychicSentry"), )); effect_was_triggered = true; }
                    SkillEffectType::FreezingNova { nova_duration_secs, slow_multiplier, slow_duration_secs, color, .. } => { let nova_spawn_position = player_transform.translation; commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/frost_nova_placeholder.png"), sprite: Sprite { custom_size: Some(Vec2::splat(0.1)), color: *color, ..default() }, transform: Transform::from_translation(nova_spawn_position.truncate().extend(0.25)), ..default() }, FreezingNovaEffect { damage: nova_damage_val, radius_sq: nova_radius_val.powi(2), lifetime_timer: Timer::from_seconds(*nova_duration_secs, TimerMode::Once), slow_multiplier: *slow_multiplier, slow_duration_secs: *slow_duration_secs, already_hit_entities: Vec::new(), }, Name::new("GlacialNovaEffect"), )); effect_was_triggered = true; sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); }
                }
                if effect_was_triggered { if let Some(skill_instance_mut) = player.equipped_skills.get_mut(idx) { skill_instance_mut.trigger(skill_def.base_cooldown); } } } }
        }
    }
}

fn survivor_buff_management_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut SurvivorBuffEffect)>,) { for (entity, mut buff) in query.iter_mut() { buff.duration_timer.tick(time.delta()); if buff.duration_timer.finished() { commands.entity(entity).remove::<SurvivorBuffEffect>(); } } } // Renamed
fn skill_projectile_lifetime_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime), With<SkillProjectile>>,) { for (entity, mut lifetime) in query.iter_mut() { lifetime.timer.tick(time.delta()); if lifetime.timer.just_finished() { commands.entity(entity).despawn_recursive(); } } }

fn skill_projectile_collision_system(
    mut commands: Commands,
    mut skill_projectile_query: Query<(Entity, &GlobalTransform, &Damage, &mut SkillProjectile, &Sprite)>, // Removed Velocity & Lifetime from here
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>, 
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    skill_library: Res<SkillLibrary>,
    player_query: Query<&Survivor>,
    glyph_library: Res<GlyphLibrary>,
) {
    let Ok(player) = player_query.get_single() else { return };

    for (proj_entity, proj_g_transform, proj_damage, mut skill_projectile_data, proj_sprite) in skill_projectile_query.iter_mut() {
        // Safety to prevent infinite loops if something goes wrong with despawning
        if skill_projectile_data.already_hit_by_this_projectile.len() > (skill_projectile_data.piercing_left + skill_projectile_data.bounces_left + 5) as usize { // Increased safety margin
             commands.entity(proj_entity).despawn_recursive();
             continue;
        }

        let proj_pos = proj_g_transform.translation().truncate();
        let proj_radius = proj_sprite.custom_size.map_or(5.0, |s| (s.x.max(s.y)) / 2.0); // Use max(s.x, s.y) for non-circular projectiles

        for (horror_entity, horror_gtransform, mut horror_health, horror_data) in horror_query.iter_mut() {
            if skill_projectile_data.already_hit_by_this_projectile.contains(&horror_entity) {
                continue;
            }
            let horror_pos = horror_gtransform.translation().truncate();
            let horror_radius = horror_data.size.x / 2.0; // Assuming circular collision for horror for now

            if proj_pos.distance(horror_pos) < proj_radius + horror_radius {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                horror_health.0 -= proj_damage.0;
                spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), proj_damage.0, &time);
                skill_projectile_data.already_hit_by_this_projectile.push(horror_entity);

                if skill_projectile_data.piercing_left > 0 {
                    skill_projectile_data.piercing_left -= 1;
                    // Projectile continues
                } else if skill_projectile_data.bounces_left > 0 {
                    skill_projectile_data.bounces_left -= 1;
                    
                    let mut closest_new_target: Option<(Entity, f32)> = None;
                    let chain_search_radius_sq = 250.0 * 250.0; // Example chain search radius

                    for (potential_target_entity, potential_target_gtransform, _health) in horror_query.iter() {
                        // Ensure not chaining to the same horror or one already hit by this specific projectile's chain sequence
                        if potential_target_entity == horror_entity || skill_projectile_data.already_hit_by_this_projectile.contains(&potential_target_entity) {
                            continue;
                        }
                        let distance_sq = potential_target_gtransform.translation().truncate().distance_squared(horror_pos); // Chain from hit horror's position
                        if distance_sq < chain_search_radius_sq {
                            if closest_new_target.is_none() || distance_sq < closest_new_target.unwrap().1 {
                                closest_new_target = Some((potential_target_entity, distance_sq));
                            }
                        }
                    }

                    if let Some((target_entity, _)) = closest_new_target {
                        if let Ok((_t_ent, target_transform, _h)) = horror_query.get(target_entity) { // Use get() for read-only access
                            let direction_to_new_target = (target_transform.translation().truncate() - horror_pos).normalize_or_zero();
                            
                            if let Some(active_skill_instance) = player.equipped_skills.iter().find(|s| s.definition_id == skill_projectile_data.skill_id) {
                                if let Some(skill_def) = skill_library.get_skill_definition(skill_projectile_data.skill_id) {
                                    if let SkillEffectType::Projectile { speed, size, color, lifetime_secs, piercing, .. } = skill_def.effect {
                                        let mut chained_damage = proj_damage.0; // Pass original damage or re-calculate with glyphs
                                        // Re-apply relevant glyphs if necessary, or assume they are part of proj_damage.0
                                        // For simplicity, let's assume proj_damage.0 already includes glyph effects from the initial cast.
                                        
                                        commands.spawn((
                                            SpriteBundle {
                                                texture: asset_server.load("sprites/eldritch_bolt_placeholder.png"),
                                                sprite: Sprite { custom_size: Some(size), color, ..default()},
                                                transform: Transform::from_translation(horror_pos.extend(proj_g_transform.translation().z))
                                                            .with_rotation(Quat::from_rotation_z(direction_to_new_target.y.atan2(direction_to_new_target.x))),
                                                ..default()
                                            },
                                            SkillProjectile {
                                                skill_id: skill_projectile_data.skill_id,
                                                piercing_left: piercing, // Reset piercing for the new chain, or use a different logic
                                                bounces_left: skill_projectile_data.bounces_left, // Pass remaining bounces
                                                already_hit_by_this_projectile: vec![target_entity], // Initialize with the new target
                                            },
                                            Velocity(direction_to_new_target * speed),
                                            Damage(chained_damage),
                                            Lifetime { timer: Timer::from_seconds(lifetime_secs, TimerMode::Once) }, // Reset lifetime for chain
                                            Name::new(format!("ChainedProjectile_{}", skill_def.name)),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    commands.entity(proj_entity).despawn_recursive(); // Despawn original after chaining attempt
                    break; 
                } else {
                    commands.entity(proj_entity).despawn_recursive();
                    break; 
                }
            }
        }
    }
}

fn active_skill_aoe_system(mut commands: Commands, time: Res<Time>, mut aoe_query: Query<(Entity, &mut ActiveSkillAoEEffect, &GlobalTransform, Option<&mut Sprite>)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health), With<Horror>>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (aoe_entity, mut aoe_effect, aoe_g_transform, opt_sprite) in aoe_query.iter_mut() { aoe_effect.lifetime_timer.tick(time.delta()); if let Some(mut sprite) = opt_sprite { let lifetime_remaining_fraction = 1.0 - aoe_effect.lifetime_timer.fraction(); let initial_alpha = sprite.color.a(); sprite.color.set_a((initial_alpha * lifetime_remaining_fraction).clamp(0.0, initial_alpha)); } if aoe_effect.lifetime_timer.finished() { commands.entity(aoe_entity).despawn_recursive(); continue; } aoe_effect.tick_timer.tick(time.delta()); if aoe_effect.tick_timer.just_finished() { aoe_effect.already_hit_this_tick.clear(); let aoe_pos = aoe_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health) in horror_query.iter_mut() { if aoe_effect.already_hit_this_tick.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(aoe_pos) < aoe_effect.actual_radius_sq { sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit)); horror_health.0 -= aoe_effect.actual_damage_per_tick; spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), aoe_effect.actual_damage_per_tick, &time); aoe_effect.already_hit_this_tick.push(horror_entity); } } } } }
fn freezing_nova_effect_damage_system( mut commands: Commands, time: Res<Time>, mut nova_query: Query<(Entity, &mut FreezingNovaEffect, &GlobalTransform, &mut Sprite, &mut Transform)>, mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &mut Velocity), (With<Horror>, Without<crate::horror::Frozen>)>, asset_server: Res<AssetServer>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (nova_entity, mut nova, nova_g_transform, mut sprite, mut vis_transform) in nova_query.iter_mut() { nova.lifetime_timer.tick(time.delta()); let progress = nova.lifetime_timer.fraction(); let current_visual_radius = nova.radius_sq.sqrt() * 2.0 * progress; vis_transform.scale = Vec3::splat(current_visual_radius); sprite.color.set_a((1.0 - progress * progress).max(0.0)); if nova.lifetime_timer.fraction() < 0.5 && !nova.already_hit_entities.contains(&nova_entity) { let nova_pos = nova_g_transform.translation().truncate(); for (horror_entity, horror_gtransform, mut horror_health, _horror_velocity) in horror_query.iter_mut() { if nova.already_hit_entities.contains(&horror_entity) { continue; } let horror_pos = horror_gtransform.translation().truncate(); if horror_pos.distance_squared(nova_pos) < nova.radius_sq { horror_health.0 -= nova.damage; spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), nova.damage, &time); sound_event_writer.send(PlaySoundEvent(SoundEffect::RitualCast)); commands.entity(horror_entity).insert(crate::horror::Frozen { timer: Timer::from_seconds(nova.slow_duration_secs, TimerMode::Once), speed_multiplier: nova.slow_multiplier, }); nova.already_hit_entities.push(horror_entity); } } if !nova.already_hit_entities.contains(&nova_entity) { nova.already_hit_entities.push(nova_entity); } } if nova.lifetime_timer.finished() { commands.entity(nova_entity).despawn_recursive(); } } }

#[cfg(test)]
mod tests {
    use super::*;
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
}