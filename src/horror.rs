use bevy::prelude::*;
use rand::{Rng, seq::SliceRandom};
use std::time::Duration; // Ensured Duration is imported
use crate::{
    components::{Velocity, Health, Damage, Lifetime},
    player::Survivor,
    game::{AppState, GameState},
    audio::{PlaySoundEvent, SoundEffect},
    items::{ItemDrop, ItemLibrary, ITEM_DROP_SIZE, ItemEffect, SurvivorTemporaryBuff, TemporaryHealthRegenBuff},
    experience::{spawn_echoing_soul, ECHOING_SOUL_VALUE},
};

#[derive(Component, Debug)]
pub struct Frozen { pub timer: Timer, pub speed_multiplier: f32, }

pub const SKITTERING_SHADOWLIMG_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const FLOATING_EYEBALL_SIZE: Vec2 = Vec2::new(40.0, 40.0);
pub const AMORPHOUS_FLESHBEAST_SIZE: Vec2 = Vec2::new(60.0, 60.0);
pub const VOID_BLINKER_SIZE: Vec2 = Vec2::new(30.0, 45.0);
pub const FLESH_WEAVER_SIZE: Vec2 = Vec2::new(45.0, 45.0);
pub const CRAWLING_TORMENT_SIZE: Vec2 = Vec2::new(25.0, 25.0);
pub const FRENZIED_BEHEMOTH_SIZE: Vec2 = Vec2::new(55.0, 50.0);

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
pub struct MaxHorrors(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorrorType {
    SkitteringShadowling, FloatingEyeball, AmorphousFleshbeast, VoidBlinker, FleshWeaver, CrawlingTorment, FrenziedBehemoth,
}

pub struct HorrorStats {
    pub horror_type: HorrorType, pub health: i32, pub damage_on_collision: i32, pub speed: f32, pub size: Vec2,
    pub sprite_path: &'static str, pub projectile_range: Option<f32>, pub projectile_fire_rate: Option<f32>,
    pub projectile_speed: Option<f32>, pub projectile_damage: Option<i32>, pub xp_value: u32,
    pub item_drop_chance_override: Option<f64>,
}

impl HorrorStats {
    fn get_for_type(horror_type: HorrorType, wave_multiplier: f32) -> Self {
        match horror_type {
            HorrorType::SkitteringShadowling => HorrorStats { horror_type, health: (20.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 10, speed: 100.0 + 20.0 * (wave_multiplier - 1.0).max(0.0), size: SKITTERING_SHADOWLIMG_SIZE, sprite_path: "sprites/skittering_shadowling_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE, item_drop_chance_override: Some(ITEM_DROP_CHANCE), },
            HorrorType::FloatingEyeball => HorrorStats { horror_type, health: (15.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 5, speed: 70.0 + 15.0 * (wave_multiplier - 1.0).max(0.0), size: FLOATING_EYEBALL_SIZE, sprite_path: "sprites/floating_eyeball_placeholder.png", projectile_range: Some(350.0), projectile_fire_rate: Some(2.8), projectile_speed: Some(280.0), projectile_damage: Some(10), xp_value: ECHOING_SOUL_VALUE + 5, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.02), },
            HorrorType::AmorphousFleshbeast => HorrorStats { horror_type, health: (60.0 * wave_multiplier * 1.5).max(1.0) as i32, damage_on_collision: 20, speed: 50.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: AMORPHOUS_FLESHBEAST_SIZE, sprite_path: "sprites/amorphous_fleshbeast_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE + 15, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.05), },
            HorrorType::VoidBlinker => HorrorStats { horror_type, health: (30.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 15, speed: 110.0 + 20.0 * (wave_multiplier - 1.0).max(0.0), size: VOID_BLINKER_SIZE, sprite_path: "sprites/void_blinker_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE + 10, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.03), },
            HorrorType::FleshWeaver => HorrorStats { horror_type, health: (40.0 * wave_multiplier * 1.2).max(1.0) as i32, damage_on_collision: 8, speed: 60.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: FLESH_WEAVER_SIZE, sprite_path: "sprites/flesh_weaver_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE + 20, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.07), },
            HorrorType::CrawlingTorment => HorrorStats { horror_type, health: (5.0 * wave_multiplier).max(1.0) as i32, damage_on_collision: 5, speed: 120.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), size: CRAWLING_TORMENT_SIZE, sprite_path: "sprites/crawling_torment_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE / 5, item_drop_chance_override: Some(MINION_ITEM_DROP_CHANCE), },
            HorrorType::FrenziedBehemoth => HorrorStats { horror_type, health: (70.0 * wave_multiplier * 1.3).max(1.0) as i32, damage_on_collision: 25, speed: 80.0 + 15.0 * (wave_multiplier - 1.0).max(0.0), size: FRENZIED_BEHEMOTH_SIZE, sprite_path: "sprites/frenzied_behemoth_placeholder.png", projectile_range: None, projectile_fire_rate: None, projectile_speed: None, projectile_damage: None, xp_value: ECHOING_SOUL_VALUE + 25, item_drop_chance_override: Some(ITEM_DROP_CHANCE + 0.1), },
        }
    }
}

#[derive(Component)]
pub struct Horror {
    pub horror_type: HorrorType, pub size: Vec2, pub damage_on_collision: i32, pub speed: f32,
    pub xp_value: u32, pub item_drop_chance: f64, pub is_elite: bool,
}

#[derive(Component)]
pub struct RangedAttackerBehavior { pub shooting_range: f32, pub fire_timer: Timer, pub projectile_speed: f32, pub projectile_damage: i32, pub state: RangedAttackerState, pub reposition_target: Option<Vec2>, pub reposition_timer: Timer, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangedAttackerState { Idle, Attacking, Repositioning, }
impl Default for RangedAttackerBehavior { fn default() -> Self { Self { shooting_range: 300.0, fire_timer: Timer::from_seconds(2.0, TimerMode::Repeating), projectile_speed: 250.0, projectile_damage: 8, state: RangedAttackerState::Idle, reposition_target: None, reposition_timer: Timer::from_seconds(REPOSITION_DURATION_SECONDS, TimerMode::Once), } } }

#[derive(Component)]
pub struct VoidBlinkerBehavior { pub state: VoidBlinkerState, pub action_timer: Timer, pub next_teleport_destination: Option<Vec2>, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoidBlinkerState { Chasing, PhasingOut, PhasedOut, PhasingIn, Cooldown, }
impl Default for VoidBlinkerBehavior { fn default() -> Self { Self { state: VoidBlinkerState::Chasing, action_timer: Timer::from_seconds(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS, TimerMode::Once), next_teleport_destination: None, } } }

#[derive(Component)]
pub struct FleshWeaverBehavior { pub summon_timer: Timer, pub max_minions: u32, pub active_minion_entities: Vec<Entity>, }
impl Default for FleshWeaverBehavior { fn default() -> Self { Self { summon_timer: Timer::from_seconds(SUMMONER_SUMMON_COOLDOWN_SECS, TimerMode::Repeating), max_minions: SUMMONER_MAX_ACTIVE_MINIONS, active_minion_entities: Vec::new(), } } }

#[derive(Component)]
pub struct FrenziedBehemothBehavior { pub state: FrenziedBehemothState, pub charge_cooldown_timer: Timer, pub telegraph_timer: Timer, pub charge_duration_timer: Timer, pub charge_target_pos: Option<Vec2>, pub charge_direction: Option<Vec2>, }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrenziedBehemothState { Roaming, Telegraphing, Charging, Cooldown, }
impl Default for FrenziedBehemothBehavior { fn default() -> Self { Self { state: FrenziedBehemothState::Roaming, charge_cooldown_timer: Timer::from_seconds(CHARGER_CHARGE_COOLDOWN_SECS, TimerMode::Once), telegraph_timer: Timer::from_seconds(CHARGER_TELEGRAPH_SECS, TimerMode::Once), charge_duration_timer: Timer::from_seconds(CHARGER_CHARGE_DURATION_SECS, TimerMode::Once), charge_target_pos: None, charge_direction: None, } } }

#[derive(Component)] pub struct HorrorProjectile;
const HORROR_PROJECTILE_SPRITE_SIZE: Vec2 = Vec2::new(15.0, 15.0);
const HORROR_PROJECTILE_COLOR: Color = Color::rgb(0.3, 0.8, 0.4);
const HORROR_PROJECTILE_LIFETIME: f32 = 3.5;
const HORROR_PROJECTILE_Z_POS: f32 = 0.7;

fn spawn_horror_projectile( commands: &mut Commands, asset_server: &Res<AssetServer>, mut position: Vec3, direction: Vec2, speed: f32, damage: i32,) {
    position.z = HORROR_PROJECTILE_Z_POS;
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/horror_ichor_blast_placeholder.png"),
            sprite: Sprite { custom_size: Some(HORROR_PROJECTILE_SPRITE_SIZE), color: HORROR_PROJECTILE_COLOR, ..default() },
            visibility: Visibility::Visible,
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        HorrorProjectile, Velocity(direction * speed), Damage(damage),
        Lifetime { timer: Timer::from_seconds(HORROR_PROJECTILE_LIFETIME, TimerMode::Once)},
        Name::new("HorrorIchorBlast"),
    ));
}

#[derive(Resource)] pub struct HorrorSpawnTimer { pub timer: Timer, }
impl Default for HorrorSpawnTimer { fn default() -> Self { Self { timer: Timer::from_seconds(2.0, TimerMode::Repeating), } } }

pub struct HorrorPlugin;
fn should_despawn_all_entities_on_session_end(next_state: Res<NextState<AppState>>) -> bool { match next_state.0 { Some(AppState::MainMenu) | Some(AppState::GameOver) => true, _ => false, } }

impl Plugin for HorrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                horror_spawn_system,
                horror_movement_system,
                frozen_effect_tick_system, // System for Frozen effect
                ranged_attacker_logic,
                void_blinker_ai_system,
                flesh_weaver_ai_system,
                frenzied_behemoth_ai_system,
                horror_projectile_collision_system,
                horror_projectile_lifetime_system,
                handle_horror_death_drops,
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(PostUpdate, update_horror_count_system_in_game_state.run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), (
                despawn_all_horrors.run_if(should_despawn_all_entities_on_session_end),
                despawn_all_item_drops.run_if(should_despawn_all_entities_on_session_end)
            ));
    }
}

pub fn despawn_all_horrors(mut commands: Commands, horror_query: Query<Entity, With<Horror>>) {
    for entity in horror_query.iter() { commands.entity(entity).despawn_recursive(); }
}
fn despawn_all_item_drops(mut commands: Commands, item_drop_query: Query<Entity, With<ItemDrop>>) {
    for entity in item_drop_query.iter() { commands.entity(entity).despawn_recursive(); }
}

fn spawn_horror_type(
    commands: &mut Commands, asset_server: &Res<AssetServer>, horror_type: HorrorType,
    position: Vec3, wave_multiplier: f32, is_elite: bool,
) {
    let base_stats = HorrorStats::get_for_type(horror_type, wave_multiplier);
    let mut final_health = base_stats.health; let mut final_damage = base_stats.damage_on_collision;
    let mut final_speed = base_stats.speed; let mut final_size = base_stats.size;
    let mut final_xp = base_stats.xp_value; let mut final_item_chance = base_stats.item_drop_chance_override.unwrap_or(0.0);
    let mut final_name = format!("{:?}", base_stats.horror_type); let mut sprite_color = Color::WHITE;

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

    let mut horror_entity_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(base_stats.sprite_path),
            sprite: Sprite { custom_size: Some(final_size), color: sprite_color, ..default() },
            transform: Transform::from_translation(position), ..default()
        },
        Horror {
            horror_type: base_stats.horror_type, size: final_size, damage_on_collision: final_damage,
            speed: final_speed, xp_value: final_xp, item_drop_chance: final_item_chance, is_elite,
        },
        Health(final_health), Velocity(Vec2::ZERO), Name::new(final_name),
    ));

    match base_stats.horror_type {
        HorrorType::FloatingEyeball => { horror_entity_commands.insert(RangedAttackerBehavior { shooting_range: base_stats.projectile_range.unwrap_or(350.0), fire_timer: Timer::from_seconds(base_stats.projectile_fire_rate.unwrap_or(2.8), TimerMode::Repeating), projectile_speed: base_stats.projectile_speed.unwrap_or(280.0), projectile_damage: base_stats.projectile_damage.unwrap_or(10), state: RangedAttackerState::Idle, reposition_target: None, reposition_timer: Timer::from_seconds(REPOSITION_DURATION_SECONDS, TimerMode::Once), }); }
        HorrorType::VoidBlinker => { horror_entity_commands.insert(VoidBlinkerBehavior::default()); }
        HorrorType::FleshWeaver => { horror_entity_commands.insert(FleshWeaverBehavior::default()); }
        HorrorType::FrenziedBehemoth => { horror_entity_commands.insert(FrenziedBehemothBehavior::default());}
        _ => {}
    }
}

fn horror_spawn_system(
    mut commands: Commands, time: Res<Time>, mut spawn_timer: ResMut<HorrorSpawnTimer>,
    asset_server: Res<AssetServer>, player_query: Query<&Transform, With<Survivor>>,
    horror_query: Query<(), With<Horror>>, max_horrors: Res<MaxHorrors>, game_state: Res<GameState>,
) {
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() || horror_query.iter().count() >= max_horrors.0 as usize { return; }
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
        1..=2 => HorrorType::SkitteringShadowling,
        3..=4 => { if rng.gen_bool(0.3) { HorrorType::SkitteringShadowling } else if rng.gen_bool(0.3) { HorrorType::FloatingEyeball } else { HorrorType::VoidBlinker } }
        5..=6 => { let roll = rng.gen_range(0..100); if roll < 20 { HorrorType::SkitteringShadowling } else if roll < 40 { HorrorType::FloatingEyeball } else if roll < 60 { HorrorType::VoidBlinker } else { HorrorType::FleshWeaver } }
        _ => { let roll = rng.gen_range(0..100); if roll < 15 { HorrorType::SkitteringShadowling } else if roll < 30 { HorrorType::FloatingEyeball } else if roll < 45 { HorrorType::VoidBlinker } else if roll < 60 { HorrorType::FleshWeaver } else if roll < 80 { HorrorType::FrenziedBehemoth } else { HorrorType::AmorphousFleshbeast } }
    };
    let is_elite = rng.gen_bool(ELITE_SPAWN_CHANCE) &&
                   chosen_type != HorrorType::CrawlingTorment &&
                   chosen_type != HorrorType::FleshWeaver && // For now, summoners and chargers don't become elite
                   chosen_type != HorrorType::FrenziedBehemoth;
    spawn_horror_type(&mut commands, &asset_server, chosen_type, final_spawn_pos, wave_multiplier, is_elite);
}

fn horror_movement_system( mut query: Query<(&mut Transform, &mut Velocity, &Horror, Option<&RangedAttackerBehavior>, Option<&VoidBlinkerBehavior>, Option<&FleshWeaverBehavior>, Option<&FrenziedBehemothBehavior>, Option<&Frozen>)>, player_query: Query<&Transform, (With<Survivor>, Without<Horror>)>, time: Res<Time>,) {
    let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate();
    for (mut transform, mut velocity, horror_data, ranged_opt, void_blinker_opt, flesh_weaver_opt, frenzied_behemoth_opt, frozen_opt) in query.iter_mut() {
        let mut current_speed_multiplier = 1.0; if let Some(frozen) = frozen_opt { current_speed_multiplier = frozen.speed_multiplier; }
        if current_speed_multiplier == 0.0 { velocity.0 = Vec2::ZERO; continue; }
        let horror_pos = transform.translation.truncate(); let mut should_chase_player_normally = true;
        if let Some(phase_behavior) = void_blinker_opt { match phase_behavior.state { VoidBlinkerState::PhasingOut | VoidBlinkerState::PhasedOut | VoidBlinkerState::PhasingIn => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } VoidBlinkerState::Cooldown => { let direction_to_player = (player_pos - horror_pos).normalize_or_zero(); velocity.0 = direction_to_player * horror_data.speed * 0.6 * current_speed_multiplier; if direction_to_player != Vec2::ZERO {transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x));} should_chase_player_normally = false; } VoidBlinkerState::Chasing => {} } }
        if should_chase_player_normally && ranged_opt.is_some() { if let Some(ranged_behavior) = ranged_opt { match ranged_behavior.state { RangedAttackerState::Attacking => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } RangedAttackerState::Repositioning => { if let Some(target_pos) = ranged_behavior.reposition_target { let dir_to_target = (target_pos - horror_pos).normalize_or_zero(); if dir_to_target != Vec2::ZERO { velocity.0 = dir_to_target * horror_data.speed * REPOSITION_SPEED_MULTIPLIER * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(dir_to_target.y.atan2(dir_to_target.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } } RangedAttackerState::Idle => {} } } }
        if let Some(_summoner_behavior) = flesh_weaver_opt { let distance_to_player = player_pos.distance(horror_pos); if distance_to_player < 250.0 { let direction_away_from_player = (horror_pos - player_pos).normalize_or_zero(); if direction_away_from_player != Vec2::ZERO { velocity.0 = direction_away_from_player * horror_data.speed * 0.5 * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_away_from_player.y.atan2(direction_away_from_player.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } else if distance_to_player > 400.0 { let direction_to_player = (player_pos - horror_pos).normalize_or_zero(); if direction_to_player != Vec2::ZERO { velocity.0 = direction_to_player * horror_data.speed * 0.5 * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x)); } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } else { velocity.0 = Vec2::ZERO; should_chase_player_normally = false; } }
        if let Some(charger_behavior) = frenzied_behemoth_opt { match charger_behavior.state { FrenziedBehemothState::Telegraphing | FrenziedBehemothState::Cooldown => { should_chase_player_normally = false; velocity.0 = Vec2::ZERO; } FrenziedBehemothState::Charging => { if let Some(charge_dir) = charger_behavior.charge_direction { velocity.0 = charge_dir * horror_data.speed * CHARGER_CHARGE_SPEED_MULTIPLIER; } else { velocity.0 = Vec2::ZERO; } should_chase_player_normally = false; } FrenziedBehemothState::Roaming => {} } }
        if should_chase_player_normally { let direction_to_player = (player_pos - horror_pos).normalize_or_zero(); if direction_to_player != Vec2::ZERO { velocity.0 = direction_to_player * horror_data.speed * current_speed_multiplier; transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x)); } else { velocity.0 = Vec2::ZERO; } }
        transform.translation.x += velocity.0.x * time.delta_seconds(); transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn frozen_effect_tick_system( mut commands: Commands, time: Res<Time>, mut frozen_query: Query<(Entity, &mut Frozen)>,) { for (entity, mut frozen_effect) in frozen_query.iter_mut() { frozen_effect.timer.tick(time.delta()); if frozen_effect.timer.finished() { commands.entity(entity).remove::<Frozen>(); } } }
fn ranged_attacker_logic(mut commands: Commands, time: Res<Time>, asset_server: Res<AssetServer>, mut attacker_query: Query<(&mut Transform, &mut RangedAttackerBehavior, &GlobalTransform, &Horror)>, player_query: Query<&Transform, (With<Survivor>, Without<Horror>)>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { let Ok(player_transform) = player_query.get_single() else { return; }; let player_position = player_transform.translation.truncate(); let mut rng = rand::thread_rng(); for (mut transform, mut behavior, attacker_gtransform, _horror_data) in attacker_query.iter_mut() { let attacker_position = attacker_gtransform.translation().truncate(); let distance_to_player = player_position.distance(attacker_position); match behavior.state { RangedAttackerState::Idle => { if distance_to_player <= behavior.shooting_range { behavior.state = RangedAttackerState::Attacking; } } RangedAttackerState::Attacking => { if distance_to_player > behavior.shooting_range * 1.1 { behavior.state = RangedAttackerState::Idle; } else { let dir = (player_position - attacker_position).normalize_or_zero(); if dir != Vec2::ZERO { transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x)); } behavior.fire_timer.tick(time.delta()); if behavior.fire_timer.just_finished() { sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorProjectile)); spawn_horror_projectile( &mut commands, &asset_server, attacker_gtransform.translation(), dir, behavior.projectile_speed, behavior.projectile_damage, ); behavior.state = RangedAttackerState::Repositioning; behavior.reposition_timer.reset(); let perp_dir = Vec2::new(-dir.y, dir.x) * (if rng.gen_bool(0.5) { 1.0 } else { -1.0 }); let dist = rng.gen_range(50.0..150.0); behavior.reposition_target = Some(attacker_position + perp_dir * dist); } } } RangedAttackerState::Repositioning => { behavior.reposition_timer.tick(time.delta()); if behavior.reposition_timer.finished() || (behavior.reposition_target.is_some() && attacker_position.distance(behavior.reposition_target.unwrap()) < 10.0) { behavior.state = RangedAttackerState::Idle; behavior.reposition_target = None; } } } } }
fn void_blinker_ai_system( _commands: Commands, time: Res<Time>, mut ripper_query: Query<(&mut Transform, &mut VoidBlinkerBehavior, &mut Sprite, &mut Visibility), (With<VoidBlinkerBehavior>, With<Horror>, Without<Survivor>)>, player_query: Query<&Transform, (With<Survivor>, Without<Horror>)>,) { let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate(); let mut rng = rand::thread_rng(); for (mut transform, mut behavior, mut sprite, mut visibility) in ripper_query.iter_mut() { behavior.action_timer.tick(time.delta()); match behavior.state { VoidBlinkerState::Chasing => { if behavior.action_timer.finished() { behavior.state = VoidBlinkerState::PhasingOut; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_PHASE_DURATION_SECS)); behavior.action_timer.reset(); let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0); let distance = rng.gen_range(PHASE_RIPPER_TELEPORT_RANGE_MIN..PHASE_RIPPER_TELEPORT_RANGE_MAX); behavior.next_teleport_destination = Some(player_pos + Vec2::new(angle.cos() * distance, angle.sin() * distance)); sprite.color.set_a(0.5); } } VoidBlinkerState::PhasingOut => { sprite.color.set_a(1.0 - behavior.action_timer.fraction()); if behavior.action_timer.just_finished() { *visibility = Visibility::Hidden; behavior.state = VoidBlinkerState::PhasedOut; behavior.action_timer.set_duration(Duration::from_millis(50)); behavior.action_timer.reset(); } } VoidBlinkerState::PhasedOut => { if behavior.action_timer.just_finished() { if let Some(destination) = behavior.next_teleport_destination.take() { transform.translation = destination.extend(transform.translation.z); } behavior.state = VoidBlinkerState::PhasingIn; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_PHASE_DURATION_SECS)); behavior.action_timer.reset(); *visibility = Visibility::Visible; sprite.color.set_a(0.0); } } VoidBlinkerState::PhasingIn => { sprite.color.set_a(behavior.action_timer.fraction()); if behavior.action_timer.just_finished() { sprite.color.set_a(1.0); behavior.state = VoidBlinkerState::Cooldown; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS)); behavior.action_timer.reset(); } } VoidBlinkerState::Cooldown => { if behavior.action_timer.finished() { behavior.state = VoidBlinkerState::Chasing; behavior.action_timer.set_duration(Duration::from_secs_f32(PHASE_RIPPER_TELEPORT_COOLDOWN_SECS)); behavior.action_timer.reset(); } } } } }
fn flesh_weaver_ai_system( mut commands: Commands, time: Res<Time>, mut summoner_query: Query<(&Transform, &mut FleshWeaverBehavior), (With<Horror>, With<FleshWeaverBehavior>)>, asset_server: Res<AssetServer>, game_state: Res<GameState>,) { let wave_multiplier = 1.0 + (game_state.wave_number as f32 - 1.0) * 0.1; for (summoner_transform, mut summoner_behavior) in summoner_query.iter_mut() { summoner_behavior.summon_timer.tick(time.delta()); summoner_behavior.active_minion_entities.retain(|&minion_e| commands.get_entity(minion_e).is_some()); if summoner_behavior.summon_timer.just_finished() && summoner_behavior.active_minion_entities.len() < summoner_behavior.max_minions as usize { for _ in 0..SUMMONER_MINIONS_TO_SPAWN { if summoner_behavior.active_minion_entities.len() >= summoner_behavior.max_minions as usize { break; } let mut rng = rand::thread_rng(); let offset_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0); let offset_distance = rng.gen_range(20.0..50.0); let spawn_offset = Vec2::new(offset_angle.cos() * offset_distance, offset_angle.sin() * offset_distance); let minion_spawn_pos = (summoner_transform.translation.truncate() + spawn_offset).extend(0.5); let minion_entity = spawn_and_return_horror_entity(&mut commands, &asset_server, HorrorType::CrawlingTorment, minion_spawn_pos, wave_multiplier); summoner_behavior.active_minion_entities.push(minion_entity); } } } }
fn spawn_and_return_horror_entity( commands: &mut Commands, asset_server: &Res<AssetServer>, horror_type: HorrorType, position: Vec3, wave_multiplier: f32,) -> Entity { let stats = HorrorStats::get_for_type(horror_type, wave_multiplier); commands.spawn(( SpriteBundle { texture: asset_server.load(stats.sprite_path), sprite: Sprite { custom_size: Some(stats.size), ..default() }, transform: Transform::from_translation(position), ..default() }, Horror { horror_type: stats.horror_type, size: stats.size, damage_on_collision: stats.damage_on_collision, speed: stats.speed, xp_value: stats.xp_value, item_drop_chance: stats.item_drop_chance_override.unwrap_or(0.0), is_elite: false }, Health(stats.health), Velocity(Vec2::ZERO), Name::new(format!("{:?}", stats.horror_type)), )).id() }
fn frenzied_behemoth_ai_system(time: Res<Time>, mut charger_query: Query<(&Transform, &mut FrenziedBehemothBehavior, &mut Sprite, &Horror)>, player_query: Query<&Transform, With<Survivor>>,){ let Ok(player_transform) = player_query.get_single() else { return; }; let player_pos = player_transform.translation.truncate(); for (charger_transform, mut behavior, mut sprite, _horror_data) in charger_query.iter_mut() { let charger_pos = charger_transform.translation.truncate(); match behavior.state { FrenziedBehemothState::Roaming => { behavior.charge_cooldown_timer.tick(time.delta()); if behavior.charge_cooldown_timer.finished() { let distance_to_player = charger_pos.distance(player_pos); if distance_to_player < CHARGER_DETECTION_RANGE && distance_to_player > CHARGER_MIN_CHARGE_RANGE { behavior.state = FrenziedBehemothState::Telegraphing; behavior.telegraph_timer.reset(); behavior.charge_target_pos = Some(player_pos); sprite.color = Color::rgb(1.0, 0.5, 0.5); } } } FrenziedBehemothState::Telegraphing => { behavior.telegraph_timer.tick(time.delta()); if behavior.telegraph_timer.just_finished() { behavior.state = FrenziedBehemothState::Charging; behavior.charge_duration_timer.reset(); if let Some(target_pos) = behavior.charge_target_pos { behavior.charge_direction = Some((target_pos - charger_pos).normalize_or_zero()); } else { behavior.charge_direction = Some((player_pos - charger_pos).normalize_or_zero()); } sprite.color = Color::rgb(1.0, 0.2, 0.2); } } FrenziedBehemothState::Charging => { behavior.charge_duration_timer.tick(time.delta()); if behavior.charge_duration_timer.finished() { behavior.state = FrenziedBehemothState::Cooldown; behavior.charge_cooldown_timer.reset(); let telegraph_timer_duration_val = behavior.telegraph_timer.duration(); behavior.telegraph_timer.tick(telegraph_timer_duration_val); behavior.charge_direction = None; sprite.color = Color::WHITE; } } FrenziedBehemothState::Cooldown => { if behavior.charge_cooldown_timer.finished() { behavior.state = FrenziedBehemothState::Roaming; } } } } }
fn horror_projectile_collision_system(mut commands: Commands, projectile_query: Query<(Entity, &GlobalTransform, &Damage), With<HorrorProjectile>>, mut player_query: Query<(&GlobalTransform, &mut Health, &mut Survivor), With<Survivor>>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { if let Ok((player_gtransform, mut player_health, mut player_component)) = player_query.get_single_mut() { for (projectile_entity, projectile_gtransform, projectile_damage) in projectile_query.iter() { let distance = projectile_gtransform.translation().truncate().distance(player_gtransform.translation().truncate()); let projectile_radius = HORROR_PROJECTILE_SPRITE_SIZE.x / 2.0; let player_radius = crate::player::PLAYER_SIZE.x / 2.0; if distance < projectile_radius + player_radius { if player_component.invincibility_timer.finished() { sound_event_writer.send(PlaySoundEvent(SoundEffect::SurvivorHit)); player_health.0 -= projectile_damage.0; player_component.invincibility_timer.reset(); } commands.entity(projectile_entity).despawn_recursive(); } } } }
fn horror_projectile_lifetime_system(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime), With<HorrorProjectile>>,) { for (entity, mut lifetime) in query.iter_mut() { lifetime.timer.tick(time.delta()); if lifetime.timer.just_finished() { commands.entity(entity).despawn_recursive(); } } }
fn handle_horror_death_drops(mut commands: Commands, dead_horrors_query: Query<(Entity, &Transform, &Health, &Horror)>, asset_server: Res<AssetServer>, mut game_state: ResMut<GameState>, item_library: Res<ItemLibrary>, mut sound_event_writer: EventWriter<PlaySoundEvent>, player_query: Query<(Entity, &Survivor)>,) { let Ok((player_entity, player_data)) = player_query.get_single() else { return }; let mut rng = rand::thread_rng(); for (entity, transform, health, horror_data) in dead_horrors_query.iter() { if health.0 <= 0 { sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorDeath)); game_state.score += horror_data.xp_value / 2; spawn_echoing_soul(&mut commands, &asset_server, transform.translation, horror_data.xp_value); if rng.gen_bool(horror_data.item_drop_chance) { if !item_library.items.is_empty() { if let Some(item_to_drop_def) = item_library.items.choose(&mut rng) { commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/eldritch_relic_placeholder.png"), sprite: Sprite { custom_size: Some(ITEM_DROP_SIZE), ..default() }, transform: Transform::from_translation(transform.translation.truncate().extend(0.4)), ..default() }, ItemDrop { item_id: item_to_drop_def.id }, Name::new(format!("ItemDrop_{}", item_to_drop_def.name)), )); } } } for item_id in player_data.collected_item_ids.iter() { if let Some(item_def) = item_library.get_item_definition(*item_id) { for effect in &item_def.effects { if let ItemEffect::OnHorrorKillTrigger { chance, effect: kill_effect_type } = effect { if rng.gen_bool((*chance).into()) { match kill_effect_type { SurvivorTemporaryBuff::HealthRegen { rate, duration_secs } => { commands.entity(player_entity).insert(TemporaryHealthRegenBuff { regen_per_second: *rate, duration_timer: Timer::from_seconds(*duration_secs, TimerMode::Once), }); } } } } } } commands.entity(entity).despawn_recursive(); } } }
fn update_horror_count_system_in_game_state(mut game_state: ResMut<crate::game::GameState>, horror_query: Query<(), With<Horror>>,) { game_state.horror_count = horror_query.iter().count() as u32; }
//Placeholder for fleshy_landscape_tile_placeholder.png if used
//The current code only uses one background tile, so background_tile2.png is not used.
