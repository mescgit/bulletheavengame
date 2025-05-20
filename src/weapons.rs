use bevy::prelude::*;
use crate::{
    survivor::Survivor, // Changed
    horror::Horror,   // Changed
    components::{Health, Damage},
    game::AppState, // GameState import removed as it was unused
    audio::{PlaySoundEvent, SoundEffect},
    visual_effects::spawn_damage_text,
};

// --- Circle of Warding Aura Weapon ---
#[derive(Component, Debug)]
pub struct CircleOfWarding {
    pub damage_tick_timer: Timer,
    pub current_radius: f32,
    pub base_damage_per_tick: i32,
    pub is_active: bool,
    pub visual_entity: Option<Entity>,
}

impl Default for CircleOfWarding {
    fn default() -> Self {
        Self {
            damage_tick_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            current_radius: 75.0,
            base_damage_per_tick: 3,
            is_active: false,
            visual_entity: None,
        }
    }
}

#[derive(Component)]
struct CircleOfWardingVisual;


// --- Swarm of Nightmares Weapon ---
const NIGHTMARE_LARVA_SPRITE_SIZE: Vec2 = Vec2::new(32.0, 32.0);
const NIGHTMARE_LARVA_DEBUG_COLOR: Color = Color::rgb(0.4, 0.8, 0.3);
const NIGHTMARE_LARVA_LOCAL_Z: f32 = 0.3;

#[derive(Component, Debug)]
pub struct SwarmOfNightmares {
    pub is_active: bool,
    pub num_larvae: u32,
    pub orbit_radius: f32,
    pub rotation_speed: f32,
    pub damage_per_hit: i32,
    pub hit_cooldown_duration: f32,
}

impl Default for SwarmOfNightmares {
    fn default() -> Self {
        Self {
            is_active: false,
            num_larvae: 0,
            orbit_radius: 80.0,
            rotation_speed: std::f32::consts::PI / 2.0,
            damage_per_hit: 5,
            hit_cooldown_duration: 0.75,
        }
    }
}

#[derive(Component)]
pub struct NightmareLarva {
    pub angle: f32,
    pub enemies_on_cooldown: Vec<(Entity, Timer)>,
}


pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            (
                circle_of_warding_aura_system,
                update_circle_of_warding_visual_system,
                manage_nightmare_larvae_system,
                nightmare_larva_movement_system,
                nightmare_larva_collision_system,
            )
            .chain()
            .run_if(in_state(AppState::InGame))
        );
        app.add_systems(PostUpdate, cleanup_aura_visuals_on_weapon_remove);
    }
}

fn circle_of_warding_aura_system(
    _commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut CircleOfWarding), With<Survivor>>,
    mut horror_query: Query<(&Transform, &mut Health, &Horror), With<Horror>>,
) {
    for (player_transform, mut aura_weapon) in player_query.iter_mut() {
        if !aura_weapon.is_active { continue; }
        aura_weapon.damage_tick_timer.tick(time.delta());
        if aura_weapon.damage_tick_timer.just_finished() {
            let player_position = player_transform.translation.truncate();
            let aura_radius_sq = aura_weapon.current_radius.powi(2);
            for (horror_transform, mut horror_health, _horror_data) in horror_query.iter_mut() {
                let horror_position = horror_transform.translation.truncate();
                if player_position.distance_squared(horror_position) < aura_radius_sq {
                    horror_health.0 -= aura_weapon.base_damage_per_tick;
                }
            }
        }
    }
}

fn update_circle_of_warding_visual_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut CircleOfWarding), With<Survivor>>,
    mut visual_query: Query<(Entity, &mut Transform, &mut Sprite), With<CircleOfWardingVisual>>,
) {
    if let Ok((player_entity, mut aura_weapon)) = player_query.get_single_mut() {
        if aura_weapon.is_active {
            let diameter = aura_weapon.current_radius * 2.0;
            let target_scale = diameter;
            if let Some(visual_entity) = aura_weapon.visual_entity {
                if let Ok((_v_ent, mut visual_transform, _visual_sprite)) = visual_query.get_mut(visual_entity) {
                    visual_transform.scale = Vec3::splat(target_scale);
                } else { aura_weapon.visual_entity = None; }
            }
            if aura_weapon.visual_entity.is_none() {
                let visual_entity = commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/circle_of_warding_effect_placeholder.png"),
                        sprite: Sprite { custom_size: Some(Vec2::splat(1.0)), color: Color::rgba(0.4, 0.2, 0.6, 0.4), ..default() },
                        transform: Transform { translation: Vec3::new(0.0, 0.0, 0.1), scale: Vec3::splat(target_scale), ..default() },
                        visibility: Visibility::Visible, ..default()
                    }, CircleOfWardingVisual, Name::new("CircleOfWardingVisual"),
                )).id();
                commands.entity(player_entity).add_child(visual_entity);
                aura_weapon.visual_entity = Some(visual_entity);
            }
        } else {
            if let Some(visual_entity) = aura_weapon.visual_entity.take() {
                if visual_query.get_mut(visual_entity).is_ok() { commands.entity(visual_entity).despawn_recursive(); }
            }
        }
    }
}

fn cleanup_aura_visuals_on_weapon_remove(
    _commands: Commands,
    _removed_aura_weapons: RemovedComponents<CircleOfWarding>,
    _visual_query: Query<Entity, With<CircleOfWardingVisual>>,
) {
    // Placeholder
}

fn manage_nightmare_larvae_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &SwarmOfNightmares), (With<Survivor>, Changed<SwarmOfNightmares>)>,
    children_query: Query<&Children>,
    larva_query: Query<Entity, With<NightmareLarva>>,
) {
    for (player_entity, weapon_stats) in player_query.iter() {
        let mut current_larva_count = 0;
        if let Ok(children) = children_query.get(player_entity) {
            for &child_entity in children.iter() { if larva_query.get(child_entity).is_ok() { current_larva_count += 1; } }
        }
        if !weapon_stats.is_active {
            if current_larva_count > 0 { if let Ok(children) = children_query.get(player_entity) { for &child_entity in children.iter() { if larva_query.get(child_entity).is_ok() { commands.entity(child_entity).despawn_recursive(); } } } }
            continue;
        }
        if current_larva_count < weapon_stats.num_larvae {
            let num_to_spawn = weapon_stats.num_larvae - current_larva_count;
            for i in 0..num_to_spawn {
                let angle_offset = (current_larva_count + i) as f32 * (2.0 * std::f32::consts::PI / weapon_stats.num_larvae.max(1) as f32);
                let initial_local_pos = Vec3::new( weapon_stats.orbit_radius * angle_offset.cos(), weapon_stats.orbit_radius * angle_offset.sin(), NIGHTMARE_LARVA_LOCAL_Z );
                let larva_entity = commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/nightmare_larva_placeholder.png"), sprite: Sprite { custom_size: Some(NIGHTMARE_LARVA_SPRITE_SIZE), color: NIGHTMARE_LARVA_DEBUG_COLOR, ..default() }, transform: Transform::from_translation(initial_local_pos), visibility: Visibility::Visible, ..default() }, NightmareLarva { angle: angle_offset, enemies_on_cooldown: Vec::new(), }, Damage(weapon_stats.damage_per_hit), Name::new(format!("NightmareLarva_{}", i)), )).id();
                commands.entity(player_entity).add_child(larva_entity);
            }
        } else if current_larva_count > weapon_stats.num_larvae {
            let num_to_despawn = current_larva_count - weapon_stats.num_larvae;
            if let Ok(children) = children_query.get(player_entity) {
                let mut despawned_count = 0;
                for &child_entity in children.iter() { if larva_query.get(child_entity).is_ok() && despawned_count < num_to_despawn { commands.entity(child_entity).despawn_recursive(); despawned_count += 1; } }
            }
        }
    }
}

fn nightmare_larva_movement_system(
    time: Res<Time>,
    player_query: Query<(Entity, &Transform), (With<Survivor>, Without<NightmareLarva>)>,
    mut larva_query: Query<(&mut NightmareLarva, &mut Transform, &Parent)>,
    weapon_stats_query: Query<&SwarmOfNightmares, With<Survivor>>,
) {
    if let Ok((player_entity, _player_transform)) = player_query.get_single() {
        if let Ok(weapon_stats) = weapon_stats_query.get(player_entity) {
            if !weapon_stats.is_active || weapon_stats.num_larvae == 0 { return; }
            for (mut larva, mut larva_transform, parent) in larva_query.iter_mut() {
                if parent.get() == player_entity {
                    larva.angle += weapon_stats.rotation_speed * time.delta_seconds(); larva.angle %= 2.0 * std::f32::consts::PI;
                    let mut local_pos = Vec3::ZERO; local_pos.x = weapon_stats.orbit_radius * larva.angle.cos(); local_pos.y = weapon_stats.orbit_radius * larva.angle.sin(); local_pos.z = NIGHTMARE_LARVA_LOCAL_Z;
                    larva_transform.translation = local_pos;
                }
            }
        }
    }
}

fn nightmare_larva_collision_system(
    mut commands: Commands,
    time: Res<Time>,
    mut larva_query: Query<(Entity, &GlobalTransform, &Damage, &mut NightmareLarva)>,
    mut horror_query: Query<(Entity, &GlobalTransform, &mut Health, &Horror)>, // Added &Horror
    asset_server: Res<AssetServer>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    player_weapon_query: Query<&SwarmOfNightmares, With<Survivor>>,
) {
    let Ok(weapon_stats) = player_weapon_query.get_single() else { return; };
    if !weapon_stats.is_active { return; }

    for (_larva_entity, larva_g_transform, larva_damage, mut larva_data) in larva_query.iter_mut() {
        larva_data.enemies_on_cooldown.retain_mut(|(_enemy_id, timer)| {
            timer.tick(time.delta()); !timer.finished()
        });
        let larva_pos = larva_g_transform.translation().truncate();
        let larva_radius = NIGHTMARE_LARVA_SPRITE_SIZE.x / 2.0;

        for (horror_entity, horror_gtransform, mut horror_health, horror_data) in horror_query.iter_mut() { // Added horror_data
            if larva_data.enemies_on_cooldown.iter().any(|(e_id, _)| *e_id == horror_entity) { continue; }
            let horror_pos = horror_gtransform.translation().truncate();
            let horror_radius = horror_data.size.x / 2.0; // Use horror_data
            if larva_pos.distance(horror_pos) < larva_radius + horror_radius {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                horror_health.0 -= larva_damage.0;
                spawn_damage_text(&mut commands, &asset_server, horror_gtransform.translation(), larva_damage.0, &time);
                larva_data.enemies_on_cooldown.push((horror_entity, Timer::from_seconds(weapon_stats.hit_cooldown_duration, TimerMode::Once)));
            }
        }
    }
}