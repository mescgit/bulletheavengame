use bevy::prelude::*;
use crate::{
    player::Player,
    enemy::Enemy,
    components::{Health, Damage}, 
    game::{AppState, GameState}, 
    audio::{PlaySoundEvent, SoundEffect},
    visual_effects::spawn_damage_text,
    experience::spawn_experience_orb,
};

// --- AoE Aura Weapon ---
#[derive(Component, Debug)]
pub struct AoeAuraWeapon {
    pub damage_tick_timer: Timer,
    pub current_radius: f32,
    pub base_damage_per_tick: i32,
    pub is_active: bool,
    pub visual_entity: Option<Entity>,
}

impl Default for AoeAuraWeapon {
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
struct AoeAuraVisual;


// --- Orbiting Projectiles Weapon ---
const ORBITER_SPRITE_SIZE: Vec2 = Vec2::new(32.0, 32.0); 
const ORBITER_DEBUG_COLOR: Color = Color::FUCHSIA; 
const ORBITER_LOCAL_Z: f32 = 0.3; 

#[derive(Component, Debug)]
pub struct OrbitingProjectileWeapon {
    pub is_active: bool,
    pub num_orbiters: u32,
    pub orbit_radius: f32,
    pub rotation_speed: f32, 
    pub damage_per_hit: i32,
    pub hit_cooldown_duration: f32, 
}

impl Default for OrbitingProjectileWeapon {
    fn default() -> Self {
        Self {
            is_active: false,
            num_orbiters: 0, 
            orbit_radius: 80.0,
            rotation_speed: std::f32::consts::PI / 2.0, 
            damage_per_hit: 5,
            hit_cooldown_duration: 0.75,
        }
    }
}

#[derive(Component)]
pub struct Orbiter {
    pub angle: f32, 
    pub enemies_on_cooldown: Vec<(Entity, Timer)>,
}


pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            (
                aoe_aura_weapon_system,
                update_aoe_aura_visual_system,
                manage_orbiters_system,
                orbiter_movement_system,
                orbiter_collision_system,
            )
            .chain()
            .run_if(in_state(AppState::InGame))
        );
        app.add_systems(PostUpdate, cleanup_aura_visuals_on_weapon_remove);
    }
}

// --- AoE Aura Systems ---
fn aoe_aura_weapon_system(
    _commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut AoeAuraWeapon), With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Health, &Enemy), With<Enemy>>, 
) {
    for (player_transform, mut aura_weapon) in player_query.iter_mut() {
        if !aura_weapon.is_active {
            continue;
        }

        aura_weapon.damage_tick_timer.tick(time.delta());

        if aura_weapon.damage_tick_timer.just_finished() {
            let player_position = player_transform.translation.truncate();
            let aura_radius_sq = aura_weapon.current_radius.powi(2);

            for (enemy_transform, mut enemy_health, _enemy_data) in enemy_query.iter_mut() {
                let enemy_position = enemy_transform.translation.truncate(); 
                if player_position.distance_squared(enemy_position) < aura_radius_sq {
                    enemy_health.0 -= aura_weapon.base_damage_per_tick;
                }
            }
        }
    }
}

fn update_aoe_aura_visual_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &mut AoeAuraWeapon), With<Player>>, 
    mut visual_query: Query<(Entity, &mut Transform, &mut Sprite), With<AoeAuraVisual>>,
) {
    if let Ok((player_entity, mut aura_weapon)) = player_query.get_single_mut() {
        if aura_weapon.is_active {
            let diameter = aura_weapon.current_radius * 2.0;
            let target_scale = diameter; 

            if let Some(visual_entity) = aura_weapon.visual_entity {
                if let Ok((_v_ent, mut visual_transform, _visual_sprite)) = visual_query.get_mut(visual_entity) {
                    visual_transform.scale = Vec3::splat(target_scale);
                } else {
                    aura_weapon.visual_entity = None;
                }
            } 
            
            if aura_weapon.visual_entity.is_none() {
                let visual_entity = commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/aura_effect.png"),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(1.0)), 
                            color: Color::rgba(0.3, 1.0, 0.3, 0.5),
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.0, 0.1), 
                            scale: Vec3::splat(target_scale), 
                            ..default()
                        },
                        visibility: Visibility::Visible,
                        ..default()
                    },
                    AoeAuraVisual,
                    Name::new("AoeAuraVisual"),
                )).id();
                commands.entity(player_entity).add_child(visual_entity);
                aura_weapon.visual_entity = Some(visual_entity);
            }
        } else {
            if let Some(visual_entity) = aura_weapon.visual_entity.take() {
                if visual_query.get_mut(visual_entity).is_ok() {
                     commands.entity(visual_entity).despawn_recursive();
                }
            }
        }
    }
}

fn cleanup_aura_visuals_on_weapon_remove(
    _commands: Commands,
    _removed_aura_weapons: RemovedComponents<AoeAuraWeapon>,
    _visual_query: Query<Entity, With<AoeAuraVisual>>,
) {
    // ...
}


// --- Orbiting Projectiles Systems ---
fn manage_orbiters_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &OrbitingProjectileWeapon), (With<Player>, Changed<OrbitingProjectileWeapon>)>, 
    children_query: Query<&Children>,
    orbiter_query: Query<Entity, With<Orbiter>>, 
) {
    // info!("[Weapons] manage_orbiters_system attempting to run.");
    for (player_entity, weapon_stats) in player_query.iter() {
        // info!("[Weapons] Managing orbiters for player {:?}. Active: {}, Target count: {}", player_entity, weapon_stats.is_active, weapon_stats.num_orbiters);

        let mut current_orbiter_count = 0;
        if let Ok(children) = children_query.get(player_entity) {
            for &child_entity in children.iter() {
                if orbiter_query.get(child_entity).is_ok() {
                    current_orbiter_count += 1;
                }
            }
        }
        // info!("[Weapons] Current orbiter count for player {:?}: {}", player_entity, current_orbiter_count);

        if !weapon_stats.is_active {
            if current_orbiter_count > 0 {
                // info!("[Weapons] Weapon inactive, despawning {} orbiters for player {:?}.", current_orbiter_count, player_entity);
                if let Ok(children) = children_query.get(player_entity) {
                    for &child_entity in children.iter() {
                        if orbiter_query.get(child_entity).is_ok() {
                            commands.entity(child_entity).despawn_recursive();
                        }
                    }
                }
            }
            continue;
        }
        
        if current_orbiter_count < weapon_stats.num_orbiters {
            let num_to_spawn = weapon_stats.num_orbiters - current_orbiter_count;
            // info!("[Weapons] Spawning {} new orbiters for player {:?}.", num_to_spawn, player_entity);
            for i in 0..num_to_spawn {
                let angle_offset = (current_orbiter_count + i) as f32 * (2.0 * std::f32::consts::PI / weapon_stats.num_orbiters.max(1) as f32);

                let initial_local_pos = Vec3::new(
                    weapon_stats.orbit_radius * angle_offset.cos(),
                    weapon_stats.orbit_radius * angle_offset.sin(), 
                    ORBITER_LOCAL_Z
                );
                // info!("[Weapons] Spawning orbiter at initial local pos: {:?}", initial_local_pos);

                let orbiter_entity = commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/guardian_seed.png"),
                        sprite: Sprite {
                            custom_size: Some(ORBITER_SPRITE_SIZE),
                            color: ORBITER_DEBUG_COLOR, 
                            ..default()
                        },
                        transform: Transform::from_translation(initial_local_pos), 
                        visibility: Visibility::Visible, 
                        ..default()
                    },
                    Orbiter {
                        angle: angle_offset,
                        enemies_on_cooldown: Vec::new(),
                    },
                    Damage(weapon_stats.damage_per_hit),
                    Name::new(format!("GuardianSeedOrbiter_{}", i)),
                )).id();
                commands.entity(player_entity).add_child(orbiter_entity);
                // info!("[Weapons] Spawned orbiter {:?} with angle {} and initial local_pos {:?}", orbiter_entity, angle_offset, initial_local_pos);
            }
        } 
        else if current_orbiter_count > weapon_stats.num_orbiters {
            let num_to_despawn = current_orbiter_count - weapon_stats.num_orbiters;
            // info!("[Weapons] Despawning {} excess orbiters for player {:?}.", num_to_despawn, player_entity);
            if let Ok(children) = children_query.get(player_entity) {
                let mut despawned_count = 0;
                for &child_entity in children.iter() {
                    if orbiter_query.get(child_entity).is_ok() && despawned_count < num_to_despawn {
                        commands.entity(child_entity).despawn_recursive();
                        despawned_count += 1;
                    }
                }
            }
        }
    }
}


fn orbiter_movement_system(
    time: Res<Time>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Orbiter>)>, 
    mut orbiter_query: Query<(&mut Orbiter, &mut Transform, &Parent)>,
    weapon_stats_query: Query<&OrbitingProjectileWeapon, With<Player>>,
) {
    // info!("[Weapons] orbiter_movement_system running");
    if let Ok((player_entity, _player_transform)) = player_query.get_single() {
        if let Ok(weapon_stats) = weapon_stats_query.get(player_entity) { 
            if !weapon_stats.is_active || weapon_stats.num_orbiters == 0 {
                return;
            }

            for (mut orbiter, mut orbiter_transform, parent) in orbiter_query.iter_mut() {
                if parent.get() == player_entity { 
                    orbiter.angle += weapon_stats.rotation_speed * time.delta_seconds();
                    orbiter.angle %= 2.0 * std::f32::consts::PI;

                    let mut local_pos = Vec3::ZERO;
                    local_pos.x = weapon_stats.orbit_radius * orbiter.angle.cos();
                    local_pos.y = weapon_stats.orbit_radius * orbiter.angle.sin();
                    local_pos.z = ORBITER_LOCAL_Z; 

                    orbiter_transform.translation = local_pos;
                    // let global_orbiter_pos = _player_transform.translation + local_pos; 
                    // info!("[Weapons] Orbiter {:?} new local_pos: {:?}, Approx Global Pos: {:?}", parent.get(), local_pos, global_orbiter_pos);
                }
            }
        } else {
            // info!("[Weapons] Orbiter movement: No OrbitingProjectileWeapon found on player {:?}", player_entity);
        }
    } else {
        // info!("[Weapons] Orbiter movement: Player not found.");
    }
}

fn orbiter_collision_system(
    mut commands: Commands,
    time: Res<Time>,
    mut orbiter_query: Query<(Entity, &GlobalTransform, &Damage, &mut Orbiter)>,
    mut enemy_query: Query<(Entity, &GlobalTransform, &mut Health, &Transform, &Enemy)>, 
    asset_server: Res<AssetServer>,
    game_time: Res<Time>, 
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    mut game_state: ResMut<GameState>,
    player_weapon_query: Query<&OrbitingProjectileWeapon, With<Player>>,
) {
    let Ok(weapon_stats) = player_weapon_query.get_single() else { return; };
    if !weapon_stats.is_active { return; }

    for (_orbiter_entity, orbiter_g_transform, orbiter_damage, mut orbiter_data) in orbiter_query.iter_mut() {
        orbiter_data.enemies_on_cooldown.retain_mut(|(_enemy_id, timer)| {
            timer.tick(time.delta());
            !timer.finished()
        });

        let orbiter_pos = orbiter_g_transform.translation().truncate();
        let orbiter_radius = ORBITER_SPRITE_SIZE.x / 2.0;

        for (enemy_entity, enemy_g_transform, mut enemy_health, enemy_l_transform, enemy_data) in enemy_query.iter_mut() {
            if orbiter_data.enemies_on_cooldown.iter().any(|(e_id, _)| *e_id == enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_g_transform.translation().truncate();
            let enemy_radius = enemy_data.size.x / 2.0;

            if orbiter_pos.distance(enemy_pos) < orbiter_radius + enemy_radius {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyHit));
                enemy_health.0 -= orbiter_damage.0;
                spawn_damage_text(&mut commands, &asset_server, enemy_g_transform.translation(), orbiter_damage.0, &game_time);

                orbiter_data.enemies_on_cooldown.push((enemy_entity, Timer::from_seconds(weapon_stats.hit_cooldown_duration, TimerMode::Once)));

                if enemy_health.0 <= 0 {
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyDeath));
                    commands.entity(enemy_entity).despawn_recursive();
                    game_state.score += 10; 
                    spawn_experience_orb(&mut commands, &asset_server, enemy_l_transform.translation, crate::experience::EXP_ORB_VALUE);
                }
            }
        }
    }
}