use bevy::{prelude::*, window::PrimaryWindow};
use std::time::Duration;
use rand::Rng;
use crate::{
    components::{Velocity, Health as ComponentHealth},
    game::{AppState, ItemCollectedEvent},
    thought_fragment::{spawn_thought_fragment, BASE_FRAGMENT_DAMAGE, BASE_FRAGMENT_SPEED},
    enemy::Enemy,
    weapons::{WardingWhispersAura, MindLarvaSwarm},
    audio::{PlaySoundEvent, SoundEffect},
    skills::{ActiveSkillInstance, SkillLibrary, SkillId, PlayerBuffEffect},
    items::{ItemId, ItemDrop, ItemLibrary, ItemEffect, RetaliationNovaEffect},
    glyphs::GlyphId,
};

pub const PLAYER_SIZE: Vec2 = Vec2::new(50.0, 50.0);
const XP_FOR_LEVEL: [u32; 10] = [100, 150, 250, 400, 600, 850, 1100, 1400, 1800, 2500];
pub const BASE_PICKUP_RADIUS: f32 = 100.0;
const PROJECTILE_SPREAD_ANGLE_DEGREES: f32 = 10.0;
pub const INITIAL_PLAYER_MAX_HEALTH: i32 = 100;
const BASE_PLAYER_SPEED: f32 = 250.0;
const ITEM_COLLECTION_RADIUS: f32 = PLAYER_SIZE.x / 2.0 + crate::items::ITEM_DROP_SIZE.x / 2.0;

#[derive(Component)] pub struct MindAffliction { pub base_fire_rate_secs: f32, pub fire_timer: Timer, }
impl Default for MindAffliction { fn default() -> Self { let base = 0.5; Self { base_fire_rate_secs: base, fire_timer: Timer::from_seconds(base, TimerMode::Repeating), } } }
pub struct PlayerPlugin;
#[derive(Component)]
pub struct Player {
    pub speed: f32, pub experience: u32, pub current_level_xp: u32, pub level: u32,
    pub aim_direction: Vec2, pub invincibility_timer: Timer,
    pub mind_fragment_damage_bonus: i32, pub mind_fragment_speed_multiplier: f32, pub mind_fragment_piercing: u32,
    pub xp_gain_multiplier: f32, pub pickup_radius_multiplier: f32, pub additional_mind_fragments: u32,
    pub max_health: i32, pub health_regen_rate: f32,
    pub equipped_skills: Vec<ActiveSkillInstance>,
    pub collected_item_ids: Vec<ItemId>,
    pub collected_glyphs: Vec<GlyphId>,
}

impl Player {
    pub fn experience_to_next_level(&self) -> u32 { if self.level == 0 { return 0; } if (self.level as usize -1) < XP_FOR_LEVEL.len() { XP_FOR_LEVEL[self.level as usize - 1] } else { XP_FOR_LEVEL.last().unwrap_or(&2500) + (self.level - XP_FOR_LEVEL.len() as u32) * 500 } }
    pub fn add_experience( &mut self, amount: u32, next_state_value: &mut NextState<AppState>, sound_event_writer: &mut EventWriter<PlaySoundEvent>,) { let actual_xp_gained = (amount as f32 * self.xp_gain_multiplier).round() as u32; self.current_level_xp += actual_xp_gained; self.experience += actual_xp_gained; while self.current_level_xp >= self.experience_to_next_level() && self.level > 0 { let needed = self.experience_to_next_level(); self.current_level_xp -= needed; self.level += 1; sound_event_writer.send(PlaySoundEvent(SoundEffect::LevelUp)); next_state_value.set(AppState::LevelUp); if next_state_value.0 == Some(AppState::LevelUp) { break; } } }
    pub fn get_effective_pickup_radius(&self) -> f32 { BASE_PICKUP_RADIUS * self.pickup_radius_multiplier }
    pub fn new_with_skills_and_items(initial_skills: Vec<ActiveSkillInstance>, initial_items: Vec<ItemId>) -> Self { Self { speed: BASE_PLAYER_SPEED, experience: 0, current_level_xp: 0, level: 1, aim_direction: Vec2::X, invincibility_timer: Timer::from_seconds(1.0, TimerMode::Once), mind_fragment_damage_bonus: 0, mind_fragment_speed_multiplier: 1.0, mind_fragment_piercing: 0, xp_gain_multiplier: 1.0, pickup_radius_multiplier: 1.0, additional_mind_fragments: 0, max_health: INITIAL_PLAYER_MAX_HEALTH, health_regen_rate: 0.0, equipped_skills: initial_skills, collected_item_ids: initial_items, collected_glyphs: Vec::new(), } }
}

fn should_despawn_player(next_state: Res<NextState<AppState>>) -> bool { match next_state.0 { Some(AppState::GameOver) | Some(AppState::MainMenu) => true, _ => false, } }
fn no_player_exists(player_query: Query<(), With<Player>>) -> bool { player_query.is_empty() }
impl Plugin for PlayerPlugin { fn build(&self, app: &mut App) { app .add_systems(OnEnter(AppState::InGame), spawn_player.run_if(no_player_exists)) .add_systems(Update, ( player_movement, player_aiming, player_shooting_system, player_health_regeneration_system, player_enemy_collision_system.before(check_player_death_system), player_invincibility_system, check_player_death_system, player_item_drop_collection_system, ).chain().run_if(in_state(AppState::InGame))) .add_systems(OnExit(AppState::InGame), despawn_player.run_if(should_despawn_player)); } }

fn spawn_player( mut commands: Commands, asset_server: Res<AssetServer>, skill_library: Res<SkillLibrary>,) {
    let mut initial_skills = Vec::new();
    // Player starts with only Eldritch Bolt (SkillId(1))
    if let Some(skill_def_bolt) = skill_library.get_skill_definition(SkillId(1)) {
        let bolt_instance = ActiveSkillInstance::new(SkillId(1), skill_def_bolt.base_glyph_slots);
        // No hardcoded glyphs anymore
        initial_skills.push(bolt_instance);
    }
    commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/player_ship_eldritch.png"), sprite: Sprite { custom_size: Some(PLAYER_SIZE), ..default() }, transform: Transform::from_xyz(0.0, 0.0, 1.0), ..default() }, Player::new_with_skills_and_items(initial_skills, Vec::new()), ComponentHealth(INITIAL_PLAYER_MAX_HEALTH), Velocity(Vec2::ZERO), MindAffliction::default(), WardingWhispersAura::default(), MindLarvaSwarm::default(), Name::new("Player (Eldritch Hero)"), ));
}
fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) { if let Ok(player_entity) = player_query.get_single() { commands.entity(player_entity).despawn_recursive(); } }
fn player_health_regeneration_system(time: Res<Time>, mut query: Query<(&Player, &mut ComponentHealth)>,) { for (player_stats, mut current_health) in query.iter_mut() { if player_stats.health_regen_rate > 0.0 && current_health.0 > 0 && current_health.0 < player_stats.max_health { let regen_amount = player_stats.health_regen_rate * time.delta_seconds(); current_health.0 = (current_health.0 as f32 + regen_amount).round() as i32; current_health.0 = current_health.0.min(player_stats.max_health); } } }
fn player_movement( keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(&Player, &mut Transform, &mut Velocity, Option<&PlayerBuffEffect>)>, time: Res<Time>,) { for (player, mut transform, mut velocity, buff_effect_opt) in query.iter_mut() { let mut direction = Vec2::ZERO; if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; } if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; } if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; } if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; } let mut current_speed = player.speed; if let Some(buff) = buff_effect_opt { current_speed *= 1.0 + buff.speed_multiplier_bonus; } velocity.0 = if direction != Vec2::ZERO { direction.normalize() * current_speed } else { Vec2::ZERO }; transform.translation.x += velocity.0.x * time.delta_seconds(); transform.translation.y += velocity.0.y * time.delta_seconds(); } }
fn player_aiming(mut player_query: Query<(&mut Player, &Transform)>, window_query: Query<&Window, With<PrimaryWindow>>, camera_query: Query<(&Camera, &GlobalTransform)>,) { if let Ok((mut player, player_transform)) = player_query.get_single_mut() { if let Ok(primary_window) = window_query.get_single() { if let Ok((camera, camera_transform)) = camera_query.get_single() { if let Some(cursor_position) = primary_window.cursor_position() { if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) { let direction_to_mouse = (world_position - player_transform.translation.truncate()).normalize_or_zero(); if direction_to_mouse != Vec2::ZERO { player.aim_direction = direction_to_mouse; } } } } } } }
fn player_shooting_system( mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut query: Query<(&Transform, &Player, &mut MindAffliction, Option<&PlayerBuffEffect>)>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (player_transform, player_stats, mut mind_affliction, buff_effect_opt) in query.iter_mut() { let mut current_fire_rate_secs = mind_affliction.base_fire_rate_secs; if let Some(buff) = buff_effect_opt { current_fire_rate_secs /= 1.0 + buff.fire_rate_multiplier_bonus; } let new_duration = Duration::from_secs_f32(current_fire_rate_secs.max(0.05)); if mind_affliction.fire_timer.duration() != new_duration { mind_affliction.fire_timer.set_duration(new_duration); } mind_affliction.fire_timer.tick(time.delta()); if mind_affliction.fire_timer.just_finished() { if player_stats.aim_direction != Vec2::ZERO { sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerShoot)); let current_damage = BASE_FRAGMENT_DAMAGE + player_stats.mind_fragment_damage_bonus; let current_speed = BASE_FRAGMENT_SPEED * player_stats.mind_fragment_speed_multiplier; let current_piercing = player_stats.mind_fragment_piercing; let total_fragments = 1 + player_stats.additional_mind_fragments; let base_angle = player_stats.aim_direction.to_angle(); for i in 0..total_fragments { let angle_offset_rad = if total_fragments > 1 { let total_spread_angle_rad = (total_fragments as f32 - 1.0) * PROJECTILE_SPREAD_ANGLE_DEGREES.to_radians(); let start_angle_rad = base_angle - total_spread_angle_rad / 2.0; start_angle_rad + (i as f32 * PROJECTILE_SPREAD_ANGLE_DEGREES.to_radians()) } else { base_angle }; let fragment_direction = Vec2::from_angle(angle_offset_rad); spawn_thought_fragment( &mut commands, &asset_server, player_transform.translation, fragment_direction, current_damage, current_speed, current_piercing, ); } } } } }
fn player_enemy_collision_system( mut commands: Commands, asset_server: Res<AssetServer>, mut player_query: Query<(Entity, &Transform, &mut ComponentHealth, &mut Player)>, enemy_query: Query<(&Transform, &Enemy)>, item_library: Res<ItemLibrary>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { if let Ok((player_entity, player_transform, mut player_health, mut player_component)) = player_query.get_single_mut() { if !player_component.invincibility_timer.finished() { return; } for (enemy_transform, enemy_stats) in enemy_query.iter() { let distance = player_transform.translation.truncate().distance(enemy_transform.translation.truncate()); let player_radius = PLAYER_SIZE.x / 2.0; let enemy_radius = enemy_stats.size.x / 2.0; if distance < player_radius + enemy_radius { if player_component.invincibility_timer.finished() { sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerHit)); player_health.0 -= enemy_stats.damage_on_collision; player_component.invincibility_timer.reset(); let mut rng = rand::thread_rng(); for item_id in player_component.collected_item_ids.iter() { if let Some(item_def) = item_library.get_item_definition(*item_id) { for effect in &item_def.effects { if let ItemEffect::OnPlayerHitRetaliate { chance, retaliation_damage, retaliation_radius, retaliation_color } = effect { if rng.gen_bool((*chance).into()) { commands.entity(player_entity).with_children(|parent| { parent.spawn(( SpriteBundle { texture: asset_server.load("sprites/aura_effect.png"), sprite: Sprite { custom_size: Some(Vec2::splat(0.1)), color: *retaliation_color, ..default() }, transform: Transform::from_xyz(0.0, 0.0, 0.3), ..default() }, RetaliationNovaEffect { damage: *retaliation_damage, radius_sq: retaliation_radius.powi(2), timer: Timer::from_seconds(0.4, TimerMode::Once), already_hit_entities: Vec::new(), }, Name::new("RetaliationNova"), )); }); } } } } } } } } } }
fn player_invincibility_system(time: Res<Time>, mut query: Query<(&mut Player, &mut Sprite, &ComponentHealth)>,) { for (mut player, mut sprite, health) in query.iter_mut() { if health.0 <= 0 { if sprite.color.a() != 1.0 { sprite.color.set_a(1.0); } continue; } if !player.invincibility_timer.finished() { player.invincibility_timer.tick(time.delta()); let alpha = (time.elapsed_seconds() * 20.0).sin() / 2.0 + 0.7; sprite.color.set_a(alpha.clamp(0.3, 1.0) as f32); } else { if sprite.color.a() != 1.0 { sprite.color.set_a(1.0); } } } }
fn check_player_death_system(player_query: Query<&ComponentHealth, With<Player>>, mut app_state_next: ResMut<NextState<AppState>>, mut sound_event_writer: EventWriter<PlaySoundEvent>, current_app_state: Res<State<AppState>>,) { if let Ok(player_health) = player_query.get_single() { if player_health.0 <= 0 && *current_app_state.get() == AppState::InGame { sound_event_writer.send(PlaySoundEvent(SoundEffect::GameOver)); app_state_next.set(AppState::GameOver); } } }
fn player_item_drop_collection_system(mut commands: Commands, player_query: Query<&Transform, With<Player>>, item_drop_query: Query<(Entity, &Transform, &ItemDrop)>, mut item_collected_event_writer: EventWriter<ItemCollectedEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { if let Ok(player_transform) = player_query.get_single() { let player_pos = player_transform.translation.truncate(); for (item_drop_entity, item_drop_transform, item_drop_data) in item_drop_query.iter() { let item_drop_pos = item_drop_transform.translation.truncate(); if player_pos.distance(item_drop_pos) < ITEM_COLLECTION_RADIUS { item_collected_event_writer.send(ItemCollectedEvent(item_drop_data.item_id)); sound_event_writer.send(PlaySoundEvent(SoundEffect::XpCollect)); commands.entity(item_drop_entity).despawn_recursive(); } } } }