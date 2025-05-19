use bevy::prelude::*;
use rand::{Rng, seq::SliceRandom};
use std::time::Duration; // Ensured Duration is imported
use crate::{
    components::{Velocity, Health, Damage, Lifetime},
    player::Player,
    game::{AppState, GameState},
    audio::{PlaySoundEvent, SoundEffect},
    items::{ItemDrop, ItemLibrary, ITEM_DROP_SIZE, ItemEffect, PlayerTemporaryBuff, TemporaryHealthRegenBuff},
    experience::{spawn_experience_orb, EXP_ORB_VALUE},
};

#[derive(Component, Debug)]
pub struct Frozen { pub timer: Timer, pub speed_multiplier: f32, }

pub const LINGERING_DREG_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const GAZING_ORB_SIZE: Vec2 = Vec2::new(40.0, 40.0);
pub const BULWARK_OF_FLESH_SIZE: Vec2 = Vec2::new(60.0, 60.0);
pub const PHASE_RIPPER_SIZE: Vec2 = Vec2::new(30.0, 45.0);
pub const BROOD_TENDER_SIZE: Vec2 = Vec2::new(45.0, 45.0);
pub const MINDLESS_SPAWN_SIZE: Vec2 = Vec2::new(25.0, 25.0);
pub const RUINOUS_CHARGER_SIZE: Vec2 = Vec2::new(55.0, 50.0);

const ITEM_DROP_CHANCE: f64 = 0.05;
const MINION_ITEM_DROP_CHANCE: f64 = 0.01;
const ELITE_ITEM_DROP_CHANCE_BONUS: f64 = 0.10;
const ELITE_SPAWN_CHANCE: f64 = 0.05;

const REPOSITION_DURATION_SECONDS: f32 = 1.5;
const REPOSITION_SPEED_MULTIPLIER: f32 = 0.7;

const PHASE_RIPPER_TELEPORT_COOLDOWN_SECS: f32 = 5.0;
const PHASE_RIPPER_PHASE_DURATION_SECS: f32 = 0.3;
const PHASE_RIPPER_TELEPORT_RANGE_MIN: f32 = 100.0;
const PHASE_RIPPER_TELEPORT_RANGE_MAX: f32 = 250.0;

const SUMMONER_SUMMON_COOLDOWN_SECS: f32 = 7.0;
const SUMMONER_MAX_ACTIVE_MINIONS: u32 = 3;
const SUMMONER_MINIONS_TO_SPAWN: u32 = 2;

const CHARGER_CHARGE_COOLDOWN_SECS: f32 = 6.0;
const CHARGER_TELEGRAPH_SECS: f32 = 1.2;
const CHARGER_CHARGE_DURATION_SECS: f32 = 1.0;
const CHARGER_CHARGE_SPEED_MULTIPLIER: f32 = 3.5;
const CHARGER_DETECTION_RANGE: f32 = 400.0;
const CHARGER_MIN_CHARGE_RANGE: f32 = 100.0;

#[derive(Resource)]
pub struct MaxEnemies(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    LingeringDreg, GazingOrb, BulwarkOfFlesh, PhaseRipper, BroodTender, MindlessSpawn, RuinousCharger,
}

pub struct EnemyStats {
    pub enemy_type: EnemyType, pub health: i32, pub damage_on_collision: i32, pub speed: f32, pub size: Vec2,
    pub sprite_path: &'static str, pub projectile_range: Option<f32>, pub projectile_fire_rate: Option<f32>,
    pub projectile_speed: Option<f32>, pub projectile_damage: Option<i32>, pub xp_value: u32,
    pub item_drop_chance_override: Option<f64>,
}

impl EnemyStats {
    fn get_for_type(enemy_type: EnemyType, wave_multiplier: f32) -> Self {
        match enemy_type {
            EnemyType::LingeringDreg => EnemyStats { enemy_type, health: (20.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 10, speed: 100.0 + 20.0 * (wave_multiplier - 1.0).max(0.0), size: LINGERING_DREG_SIZE, sprite_path: "sprites/lingering_dreg.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE, item_drop_chance_override: Some(ITEM_DROP_CHANCE), },
            EnemyType::GazingOrb => EnemyStats { enemy_type, health: (15.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 5, speed: 70.0 + 15.0 * (wave_multiplier - 1.0).max(0.0), size: GAZING_ORB_SIZE, sprite_path: "sprites/gazing_orb.png", projectile_range: Some(350.0), projectile_fire_rate: Some(2.8), projectile_speed: Some(280.0), projectile_damage: Some(10), xp_value: EXP_ORB_VALUE + 5, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.02), },
            EnemyType::BulwarkOfFlesh => EnemyStats { enemy_type, health: (60.0 * wave_multiplier * 1.5).max(1.0) as i32, damage_on_collision: 20, speed: 50.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: BULWARK_OF_FLESH_SIZE, sprite_path: "sprites/bulwark_of_flesh.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE + 15, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.05), },
            EnemyType::PhaseRipper => EnemyStats { enemy_type, health: (30.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 15, speed: 110.0 + 20.0 * (wave_multiplier - 1.0).max(0.0), size: PHASE_RIPPER_SIZE, sprite_path: "sprites/phase_ripper.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE + 10, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.03), },
            EnemyType::BroodTender => EnemyStats { enemy_type, health: (40.0 * wave_multiplier * 1.2).max(1.0) as i32, damage_on_collision: 8, speed: 60.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: BROOD_TENDER_SIZE, sprite_path: "sprites/brood_tender.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE + 20, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.07), },
            EnemyType::MindlessSpawn => EnemyStats { enemy_type, health: (5.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 5, speed: 120.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: MINDLESS_SPAWN_SIZE, sprite_path: "sprites/mindless_spawn.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE / 5, item_drop_chance_override: Some(MINION_ITEM_DROP_CHANCE), },
            EnemyType::RuinousCharger => EnemyStats { enemy_type, health: (70.0 * wave_multiplier * 1.3).max(1.0) as i32, damage_on_collision: 25, speed: 80.0 + 15.0 * (wave_multiplier - 1.0).max(0.0), size: RUINOUS_CHARGER_SIZE, sprite_path: "sprites/ruinous_charger.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: EXP_ORB_VALUE + 25, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.1), },
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType, pub size: Vec2, pub damage_on_collision: i32, pub speed: f32,
    pub xp_value: u32, pub item_drop_chance: f64, pub is_elite: bool,
}

#[derive(Component)]
pub struct RangedAttackerBehavior { pub shooting_range: f32, pub fire_timer: Timer, pub projectile_speed: f32, pub projectile_damage: i32, pub state: RangedAttackerState, pub reposition_target: Option<Vec2>, pub reposition_timer: Timer, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangedAttackerState { Idle, Attacking, Repositioning, }
impl Default for RangedAttackerBehavior { fn default() -> Self { Self { shooting_range: 300.0, fire_timer: Timer::from_seconds(2.0, TimerMode::Repeating), projectile_speed: 250.0, projectile_damage: 8, state: RangedAttackerState::Idle, reposition_target: None, reposition_timer: Timer::from_seconds(REPOSITION_DURATION_SECONDS, TimerMode::Once), } } }

#[derive(Component)]
pub struct PhaseRipperBehavior { pub state: PhaseRipperState, pub action_timer: Timer, pub next_teleport_destination: Option<Vec2>, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseRipperState { Chasing, PhasingOut, PhasedOut, PhasingIn, Cooldown, }
impl Default for PhaseRipperBehavior { fn default() -> Self { Self { state: PhaseRipperState::Chasing, action_timer: Timer::from_seconds(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS, TimerMode::Once), next_teleport_destination: None, } } }

#[derive(Component)]
pub struct SummonerBehavior { pub summon_timer: Timer, pub max_minions: u32, pub active_minion_entities: Vec<Entity>, }
impl Default for SummonerBehavior { fn default() -> Self { Self { summon_timer: Timer::from_seconds(SUMMONER_SUMMON_COOLDOWN_SECS, TimerMode::Repeating), max_minions: SUMMONER_MAX_ACTIVE_MINIONS, active_minion_entities: Vec::new(), } } }

#[derive(Component)]
pub struct ChargerBehavior { pub state: ChargerState, pub charge_cooldown_timer: Timer, pub telegraph_timer: Timer, pub charge_duration_timer: Timer, pub charge_target_pos: Option<Vec2>, pub charge_direction: Option<Vec2>, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargerState { Roaming, Telegraphing, Charging, Cooldown, }
impl Default for ChargerBehavior { fn default() -> Self { Self { state: ChargerState::Roaming, charge_cooldown_timer: Timer::from_seconds(CHARGER_CHARGE_COOLDOWN_SECS, TimerMode::Once), telegraph_timer: Timer::from_seconds(CHARGER_TELEGRAPH_SECS, TimerMode::Once), charge_duration_timer: Timer::from_seconds(CHARGER_CHARGE_DURATION_SECS, TimerMode::Once), charge_target_pos: None, charge_direction: None, } } }

#[derive(Component)] pub struct EnemyProjectile;
const ENEMY_PROJECTILE_SPRITE_SIZE: Vec2 = Vec2::new(15.0, 15.0);
const ENEMY_PROJECTILE_COLOR: Color = Color::rgb(0.3, 0.8, 0.4);
const ENEMY_PROJECTILE_LIFETIME: f32 = 3.5;
const ENEMY_PROJECTILE_Z_POS: f32 = 0.7;

fn spawn_enemy_projectile( commands: &mut Commands, asset_server: &Res<AssetServer>, mut position: Vec3, direction: Vec2, speed: f32, damage: i32,) {
    position.z = ENEMY_PROJECTILE_Z_POS;
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/enemy_ichor_blast.png"),
            sprite: Sprite { custom_size: Some(ENEMY_PROJECTILE_SPRITE_SIZE), color: ENEMY_PROJECTILE_COLOR, ..default() },
            visibility: Visibility::Visible,
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        EnemyProjectile, Velocity(direction * speed), Damage(damage),
        Lifetime { timer: Timer::from_seconds(ENEMY_PROJECTILE_LIFETIME, TimerMode::Once)},
        Name::new("EnemyIchorBlast"),
    ));
}

#[derive(Resource)] pub struct EnemySpawnTimer { pub timer: Timer, }
impl Default for EnemySpawnTimer { fn default() -> Self { Self { timer: Timer::from_seconds(2.0, TimerMode::Repeating), } } }

pub struct EnemyPlugin;
fn should_despawn_all_entities_on_session_end(next_state: Res<NextState<AppState>>) -> bool { match next_state.0 { Some(AppState::MainMenu) | Some(AppState::GameOver) => true, _ => false, } }

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                enemy_spawn_system,
                enemy_movement_system,
                frozen_effect_tick_system, // System for Frozen effect
                ranged_attacker_logic,
                phase_ripper_ai_system,
                summoner_ai_system,
                charger_ai_system,
                enemy_projectile_collision_system,
                enemy_projectile_lifetime_system,
                handle_enemy_death_drops,
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(PostUpdate, update_enemy_count_system_in_game_state.run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), (
                despawn_all_enemies.run_if(should_despawn_all_entities_on_session_end),
                despawn_all_item_drops.run_if(should_despawn_all_entities_on_session_end)
            ));
    }
}

pub fn despawn_all_enemies(mut commands: Commands, enemy_query: Query<Entity, With<Enemy>>) {
    for entity in enemy_query.iter() { commands.entity(entity).despawn_recursive(); }
}
fn despawn_all_item_drops(mut commands: Commands, item_drop_query: Query<Entity, With<ItemDrop>>) {
    for entity in item_drop_query.iter() { commands.entity(entity).despawn_recursive(); }
}

fn spawn_enemy_type(
    commands: &mut Commands, asset_server: &Res<AssetServer>, enemy_type: EnemyType,
    position: Vec3, wave_multiplier: f32, is_elite: bool,
) {
    let base_stats = EnemyStats::get_for_type(enemy_type, wave_multiplier);
    let mut final_health = base_stats.health; let mut final_damage = base_stats.damage_on_collision;
    let mut final_speed = base_stats.speed; let mut final_size = base_stats.size;
    let mut final_xp = base_stats.xp_value; let mut final_item_chance = base_stats.item_drop_chance_override.unwrap_or(0.0);
    let mut final_name = format!("{:?}", base_stats.enemy_type); let mut sprite_color = Color::WHITE;

    if is_elite {
        final_health = (final_health as f32 * 2.5).ceil() as i32;
        final_damage = (final_damage as f32 * 1.8).ceil() as i32;
        final_speed *= 1.15;
        final_size *= 1.25;
        final_xp = (final_xp as f32 * 2.0).ceil() as u32;
        final_item_chance = (final_item_chance + ELITE_ITEM_DROP_CHANCE_BONUS).min(1.0);
        final_name = format!("[Elite] {}", final_name);
        sprite_color = Color::rgb(1.0, 0.6, 0.6);
    }

    let mut enemy_entity_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(base_stats.sprite_path),
            sprite: Sprite { custom_size: Some(final_size), color: sprite_color, ..default() },
            transform: Transform::from_translation(position), ..default()
        },
        Enemy {
            enemy_type: base_stats.enemy_type, size: final_size, damage_on_collision: final_damage,
            speed: final_speed, xp_value: final_xp, item_drop_chance: final_item_chance, is_elite,
        },
        Health(final_health), Velocity(Vec2::ZERO), Name::new(final_name),
    ));

    match base_stats.enemy_type {
        EnemyType::GazingOrb => { enemy_entity_commands.insert(RangedAttackerBehavior { shooting_range: base_stats.projectile_range.unwrap_or(350.0), fire_timer: Timer::from_seconds(base_stats.projectile_fire_rate.unwrap_or(2.8), TimerMode::Repeating), projectile_speed: base_stats.projectile_speed.unwrap_or(280.0), projectile_damage: base_stats.projectile_damage.unwrap_or(10), state: RangedAttackerState::Idle, reposition_target: None, reposition_timer: Timer::from_seconds(REPOSITION_DURATION_SECONDS, TimerMode::Once), }); }
        EnemyType::PhaseRipper => { enemy_entity_commands.insert(PhaseRipperBehavior::default()); }
        EnemyType::BroodTender => { enemy_entity_commands.insert(SummonerBehavior::default()); }
        EnemyType::RuinousCharger => { enemy_entity_commands.insert(ChargerBehavior::default());}
        _ => {}
    }
}

fn enemy_spawn_system(
    mut commands: Commands, time: Res<Time>, mut spawn_timer: ResMut<EnemySpawnTimer>,
    asset_server: Res<AssetServer>, player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(), With<Enemy>>, max_enemies: Res<MaxEnemies>, game_state: Res<GameState>,
) {
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() || enemy_query.iter().count() >= max_enemies.0 as usize { return; }
    let Ok(player_transform) = player_query.get_single() else { return; };
    let player_pos = player_transform.translation.truncate();
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
    let distance = rng.gen_range(crate::game::SCREEN_WIDTH * 0.7 .. crate::game::SCREEN_WIDTH * 1.0);
    let relative_spawn_pos = Vec2::new(angle.cos() * distance, angle.sin() * distance);
    let spawn_pos = player_pos + relative_spawn_pos;
    let final_spawn_pos = Vec3::new(spawn_pos.x, spawn_pos.y, 0.5);
    let wave_multiplier = 1.0 + (game_state.wave_number as f32 - 1.0) * 0.1;

    let chosen_type = match game_state.wave_number {
        1..=2 => EnemyType::LingeringDreg,
        3..=4 => { if rng.gen_bool(0.3) { EnemyType::LingeringDreg } else if rng.gen_bool(0.3) { EnemyType::GazingOrb } else { EnemyType::PhaseRipper } }
        5..=6 => { let roll = rng.gen_range(0..100); if roll < 20 { EnemyType::LingeringDreg } else if roll < 40 { EnemyType::GazingOrb } else if roll < 60 { EnemyType::PhaseRipper } else { EnemyType::BroodTender } }
        _ => { let roll = rng.gen_range(0..100); if roll < 15 { EnemyType::LingeringDreg } else if roll < 30 { EnemyType::GazingOrb } else if roll < 45 { EnemyType::PhaseRipper } else if roll < 60 { EnemyType::BroodTender } else if roll < 80 { EnemyType::RuinousCharger } else { EnemyType::BulwarkOfFlesh } }
    };
    let is_elite = rng.gen_bool(ELITE_SPAWN_CHANCE) &&
                   chosen_type != EnemyType::MindlessSpawn &&
                   chosen_type != EnemyType::BroodTender && // For now, summoners and chargers don't become elite
                   chosen_type != EnemyType::RuinousCharger;
    spawn_enemy_type(&mut commands, &asset_server, chosen_type, final_spawn_pos, wave_multiplier, is_elite);
}

fn enemy_movement_system( mut query: Query<(&mut Transform, &mut Velocity, &Enemy, Option<&RangedAttackerBehavior>, Option<&PhaseRipperBehavior>, Option<&SummonerBehavior>, Option<&ChargerBehavior>, Option<&Frozen>)>, player_query: Query<&Transform, (With<Player>, Without<Enemy>)>, time: Res<Time>,) {
    let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate();
    for (mut transform, mut velocity, enemy_data, ranged_opt, phase_ripper_opt, summoner_opt, charger_opt, frozen_opt) in query.iter_mut() {
        let mut current_speed_multiplier = 1.0; if let Some(frozen) = frozen_opt { current_speed_multiplier = frozen.speed_multiplier; }
        if current_speed_multiplier == 0.0 { velocity.0 = Vec2::ZERO; continue; }
        let enemy_pos = transform.translation.truncate(); let mut should_chase_player_normally = true;
        if let Some(phase_behavior) = phase_ripper_opt { match phase_behavior.state { PhaseRipperState::PhasingOut | PhaseRipperState::PhasedOut | PhaseRipperState::PhasingIn => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } PhaseRipperState::Cooldown => { let direction_to_player = (player_pos - enemy_pos).normalize_or_zero(); velocity.0 = direction_to_player * enemy_data.speed * 0.6 * current_speed_multiplier; if direction_to_player != Vec2::ZERO {transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x));} should_chase_player_normally = false; } PhaseRipperState::Chasing => {} } }
        if should_chase_player_normally && ranged_opt.is_some() { if let Some(ranged_behavior) = ranged_opt { match ranged_behavior.state { RangedAttackerState::Attacking => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } RangedAttackerState::Repositioning => { if let Some(target_pos) = ranged_behavior.reposition_target { let dir_to_target = (target_pos - enemy_pos).normalize_or_zero(); if dir_to_target != Vec2::ZERO { velocity.0 = dir_to_target * enemy_data.speed * REPOSITION_SPEED_MULTIPLIER * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(dir_to_target.y.atan2(dir_to_target.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } } RangedAttackerState::Idle => {} } } }
        if let Some(_summoner_behavior) = summoner_opt { let distance_to_player = player_pos.distance(enemy_pos); if distance_to_player < 250.0 { let direction_away_from_player = (enemy_pos - player_pos).normalize_or_zero(); if direction_away_from_player != Vec2::ZERO { velocity.0 = direction_away_from_player * enemy_data.speed * 0.5 * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_away_from_player.y.atan2(direction_away_from_player.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } else if distance_to_player > 400.0 { let direction_to_player = (player_pos - enemy_pos).normalize_or_zero(); if direction_to_player != Vec2::ZERO { velocity.0 = direction_to_player * enemy_data.speed * 0.5 * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } else { velocity.0 = Vec2::ZERO; should_chase_player_normally = false; } }
        if let Some(charger_behavior) = charger_opt { match charger_behavior.state { ChargerState::Telegraphing | ChargerState::Cooldown => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } ChargerState::Charging => { if let Some(charge_dir) = charger_behavior.charge_direction { velocity.0 = charge_dir * enemy_data.speed * CHARGER_CHARGE_SPEED_MULTIPLIER; } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } ChargerState::Roaming => {} } }
        if should_chase_player_normally { let direction_to_player = (player_pos - enemy_pos).normalize_or_zero(); if direction_to_player != Vec2::ZERO { velocity.0 = direction_to_player * enemy_data.speed * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x)); } else { velocity.0 = Vec2::ZERO; } }
        transform.translation.x += velocity.0.x * time.delta_seconds(); transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn frozen_effect_tick_system( mut commands: Commands, time: Res<Time>, mut frozen_query: Query<(Entity, &mut Frozen)>,) { for (entity, mut frozen_effect) in frozen_query.iter_mut() { frozen_effect.timer.tick(time.delta()); if frozen_effect.timer.finished() { commands.entity(entity).remove::<Frozen>(); } } }
fn ranged_attacker_logic(mut commands: Commands, time: Res<Time>, asset_server: Res<AssetServer>, mut attacker_query: Query<(&mut Transform, &mut RangedAttackerBehavior, &GlobalTransform, &Enemy)>, player_query: Query<&Transform, (With<Player>, Without<Enemy>)>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { let Ok(player_transform) = player_query.get_single() else { return; }; let player_position = player_transform.translation.truncate(); let mut rng = rand::thread_rng(); for (mut transform, mut behavior, attacker_gtransform, _enemy_data) in attacker_query.iter_mut() { let attacker_position = attacker_gtransform.translation().truncate(); let distance_to_player = player_position.distance(attacker_position); match behavior.state { RangedAttackerState::Idle => { if distance_to_player <= behavior.shooting_range { behavior.state = RangedAttackerState::Attacking; } } RangedAttackerState::Attacking => { if distance_to_player > behavior.shooting_range * 1.1 { behavior.state = RangedAttackerState::Idle; } else { let dir = (player_position - attacker_position).normalize_or_zero(); if dir != Vec2::ZERO { transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x)); } behavior.fire_timer.tick(time.delta()); if behavior.fire_timer.just_finished() { sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyShoot)); spawn_enemy_projectile( &mut commands, &asset_server, attacker_gtransform.translation(), dir, behavior.projectile_speed, behavior.projectile_damage, ); behavior.state = RangedAttackerState::Repositioning; behavior.reposition_timer.reset(); let perp_dir = Vec2::new(-dir.y, dir.x) * (if rng.gen_bool(0.5) { 1.0 } else { -1.0 }); let dist = rng.gen_range(50.0..150.0); behavior.reposition_target = Some(attacker_position + perp_dir * dist); } } } RangedAttackerState::Repositioning => { behavior.reposition_timer.tick(time.delta()); if behavior.reposition_timer.finished() || (behavior.reposition_target.is_some() && attacker_position.distance(behavior.reposition_target.unwrap()) < 10.0) { behavior.state = RangedAttackerState::Idle; behavior.reposition_target = None; } } } } }
fn phase_ripper_ai_system( _commands: Commands, time: Res<Time>, mut ripper_query: Query<(&mut Transform, &mut PhaseRipperBehavior, &mut Sprite, &mut Visibility), (With<PhaseRipperBehavior>, With<Enemy>, Without<Player>)>, player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,) { let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate(); let mut rng = rand::thread_rng(); for (mut transform, mut behavior, mut sprite, mut visibility) in ripper_query.iter_mut() { behavior.action_timer.tick(time.delta()); match behavior.state { PhaseRipperState::Chasing => { if behavior.action_timer.finished() { behavior.state = PhaseRipperState::PhasingOut; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_PHASE_DURATION_SECS)); behavior.action_timer.reset(); let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0); let distance = rng.gen_range(PHASE_RIPPER_TELEPORT_RANGE_MIN..PHASE_RIPPER_TELEPORT_RANGE_MAX); behavior.next_teleport_destination = Some(player_pos + Vec2::new(angle.cos() * distance, angle.sin() * distance)); sprite.color.set_a(0.5); } } PhaseRipperState::PhasingOut => { sprite.color.set_a(1.0 - behavior.action_timer.fraction()); if behavior.action_timer.just_finished() { *visibility = Visibility::Hidden; behavior.state = PhaseRipperState::PhasedOut; behavior.action_timer.set_duration(Duration::from_millis(50)); behavior.action_timer.reset(); } } PhaseRipperState::PhasedOut => { if behavior.action_timer.just_finished() { if let Some(destination) = behavior.next_teleport_destination.take() { transform.translation = destination.extend(transform.translation.z); } behavior.state = PhaseRipperState::PhasingIn; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_PHASE_DURATION_SECS)); behavior.action_timer.reset(); *visibility = Visibility::Visible; sprite.color.set_a(0.0); } } PhaseRipperState::PhasingIn => { sprite.color.set_a(behavior.action_timer.fraction()); if behavior.action_timer.just_finished() { sprite.color.set_a(1.0); behavior.state = PhaseRipperState::Cooldown; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS)); behavior.action_timer.reset(); } } PhaseRipperState::Cooldown => { if behavior.action_timer.finished() { behavior.state = PhaseRipperState::Chasing; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS)); behavior.action_timer.reset(); } } } } }
fn summoner_ai_system( mut commands: Commands, time: Res<Time>, mut summoner_query: Query<(&Transform, &mut SummonerBehavior), (With<Enemy>, With<SummonerBehavior>)>, asset_server: Res<AssetServer>, game_state: Res<GameState>,) { let wave_multiplier = 1.0 + (game_state.wave_number as f32 - 1.0) * 0.1; for (summoner_transform, mut summoner_behavior) in summoner_query.iter_mut() { summoner_behavior.summon_timer.tick(time.delta()); summoner_behavior.active_minion_entities.retain(|&minion_e| commands.get_entity(minion_e).is_some()); if summoner_behavior.summon_timer.just_finished() && summoner_behavior.active_minion_entities.len() < summoner_behavior.max_minions as usize { for _ in 0..SUMMONER_MINIONS_TO_SPAWN { if summoner_behavior.active_minion_entities.len() >= summoner_behavior.max_minions as usize { break; } let mut rng = rand::thread_rng(); let offset_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0); let offset_distance = rng.gen_range(20.0..50.0); let spawn_offset = Vec2::new(offset_angle.cos() * offset_distance, offset_angle.sin() * offset_distance); let minion_spawn_pos = (summoner_transform.translation.truncate() + spawn_offset).extend(0.5); let minion_entity = spawn_and_return_enemy_entity(&mut commands, &asset_server, EnemyType::MindlessSpawn, minion_spawn_pos, wave_multiplier); summoner_behavior.active_minion_entities.push(minion_entity); } } } }
fn spawn_and_return_enemy_entity( commands: &mut Commands, asset_server: &Res<AssetServer>, enemy_type: EnemyType, position: Vec3, wave_multiplier: f32,) -> Entity { let stats = EnemyStats::get_for_type(enemy_type, wave_multiplier); commands.spawn(( SpriteBundle { texture: asset_server.load(stats.sprite_path), sprite: Sprite { custom_size: Some(stats.size), ..default() }, transform: Transform::from_translation(position), ..default() }, Enemy { enemy_type: stats.enemy_type, size: stats.size, damage_on_collision: stats.damage_on_collision, speed: stats.speed, xp_value: stats.xp_value, item_drop_chance: stats.item_drop_chance_override.unwrap_or(0.0), is_elite: false }, Health(stats.health), Velocity(Vec2::ZERO), Name::new(format!("{:?}", stats.enemy_type)), )).id() }
fn charger_ai_system(time: Res<Time>, mut charger_query: Query<(&Transform, &mut ChargerBehavior, &mut Sprite, &Enemy)>, player_query: Query<&Transform, With<Player>>,){ let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate(); for (charger_transform, mut behavior, mut sprite, _enemy_data) in charger_query.iter_mut() { let charger_pos = charger_transform.translation.truncate(); match behavior.state { ChargerState::Roaming => { behavior.charge_cooldown_timer.tick(time.delta()); if behavior.charge_cooldown_timer.finished() { let distance_to_player = charger_pos.distance(player_pos); if distance_to_player < CHARGER_DETECTION_RANGE && distance_to_player > CHARGER_MIN_CHARGE_RANGE { behavior.state = ChargerState::Telegraphing; behavior.telegraph_timer.reset(); behavior.charge_target_pos = Some(player_pos); sprite.color = Color::rgb(1.0, 0.5, 0.5); } } } ChargerState::Telegraphing => { behavior.telegraph_timer.tick(time.delta()); if behavior.telegraph_timer.just_finished() { behavior.state = ChargerState::Charging; behavior.charge_duration_timer.reset(); if let Some(target_pos) = behavior.charge_target_pos { behavior.charge_direction = Some((target_pos - charger_pos).normalize_or_zero()); } else { behavior.charge_direction = Some((player_pos - charger_pos).normalize_or_zero()); } sprite.color = Color::rgb(1.0, 0.2, 0.2); } } ChargerState::Charging => { behavior.charge_duration_timer.tick(time.delta()); if behavior.charge_duration_timer.finished() { behavior.state = ChargerState::Cooldown; behavior.charge_cooldown_timer.reset(); let telegraph_timer_duration_val = behavior.telegraph_timer.duration(); behavior.telegraph_timer.tick(telegraph_timer_duration_val); behavior.charge_direction = None; sprite.color = Color::WHITE; } } ChargerState::Cooldown => { if behavior.charge_cooldown_timer.finished() { behavior.state = ChargerState::Roaming; } } } } }
fn enemy_projectile_collision_system(mut commands: Commands, projectile_query: Query<(Entity, &GlobalTransform, &Damage), With<EnemyProjectile>>, mut player_query: Query<(&GlobalTransform, &mut Health, &mut Player), With<Player>>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { if let Ok((player_gtransform, mut player_health, mut player_component)) = player_query.get_single_mut() { for (projectile_entity, projectile_gtransform, projectile_damage) in projectile_query.iter() { let distance = projectile_gtransform.translation().truncate().distance(player_gtransform.translation().truncate()); let projectile_radius = ENEMY_PROJECTILE_SPRITE_SIZE.x / 2.0; let player_radius = crate::player::PLAYER_SIZE.x / 2.0; if distance < projectile_radius + player_radius { if player_component.invincibility_timer.finished() { sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerHit)); player_health.0 -= projectile_damage.0; player_component.invincibility_timer.reset(); } commands.entity(projectile_entity).despawn_recursive(); } } } }
fn enemy_projectile_lifetime_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime), With<EnemyProjectile>>,) { for (entity, mut lifetime) in query.iter_mut() { lifetime.timer.tick(time.delta()); if lifetime.timer.just_finished() { commands.entity(entity).despawn_recursive(); } } }
fn handle_enemy_death_drops(mut commands: Commands, dead_enemies_query: Query<(Entity, &Transform, &Health, &Enemy)>, asset_server: Res<AssetServer>, mut game_state: ResMut<GameState>, item_library: Res<ItemLibrary>, mut sound_event_writer: EventWriter<PlaySoundEvent>, player_query: Query<(Entity, &Player)>,) { let Ok((player_entity, player_data)) = player_query.get_single() else { return }; let mut rng = rand::thread_rng(); for (entity, transform, health, enemy_data) in dead_enemies_query.iter() { if health.0 <= 0 { sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyDeath)); game_state.score += enemy_data.xp_value / 2; spawn_experience_orb(&mut commands, &asset_server, transform.translation, enemy_data.xp_value); if rng.gen_bool(enemy_data.item_drop_chance) { if !item_library.items.is_empty() { if let Some(item_to_drop_def) = item_library.items.choose(&mut rng) { commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/item_drop_placeholder.png"), sprite: Sprite { custom_size: Some(ITEM_DROP_SIZE), ..default() }, transform: Transform::from_translation(transform.translation.truncate().extend(0.4)), ..default() }, ItemDrop { item_id: item_to_drop_def.id }, Name::new(format!("ItemDrop_{}", item_to_drop_def.name)), )); } } } for item_id in player_data.collected_item_ids.iter() { if let Some(item_def) = item_library.get_item_definition(*item_id) { for effect in &item_def.effects { if let ItemEffect::OnEnemyKillTrigger { chance, effect: kill_effect_type } = effect { if rng.gen_bool((*chance).into()) { match kill_effect_type { PlayerTemporaryBuff::HealthRegen { rate, duration_secs } => { commands.entity(player_entity).insert(TemporaryHealthRegenBuff { regen_per_second: *rate, duration_timer: Timer::from_seconds(*duration_secs, TimerMode::Once), }); } } } } } } } commands.entity(entity).despawn_recursive(); } } }
fn update_enemy_count_system_in_game_state(mut game_state: ResMut<crate::game::GameState>, enemy_query: Query<(), With<Enemy>>,) { game_state.enemy_count = enemy_query.iter().count() as u32; }