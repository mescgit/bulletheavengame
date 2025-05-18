use bevy::{prelude::*, window::PrimaryWindow};
use crate::{
    components::{Velocity, Health as ComponentHealth},
    game::AppState, 
    bullet::{spawn_bullet, BasicWeapon, BASE_BULLET_DAMAGE, BASE_BULLET_SPEED},
    enemy::Enemy,
    weapons::AoeAuraWeapon, // Import new weapon components
    weapons::OrbitingProjectileWeapon, // Import new weapon components
    audio::{PlaySoundEvent, SoundEffect}, 
};

pub const PLAYER_SIZE: Vec2 = Vec2::new(50.0, 50.0);
const XP_FOR_LEVEL: [u32; 10] = [100, 150, 250, 400, 600, 850, 1100, 1400, 1800, 2500];
pub const BASE_PICKUP_RADIUS: f32 = 100.0; 
const PROJECTILE_SPREAD_ANGLE_DEGREES: f32 = 10.0;
pub const INITIAL_PLAYER_MAX_HEALTH: i32 = 100;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub experience: u32,
    pub current_level_xp: u32,
    pub level: u32,
    pub aim_direction: Vec2,
    pub invincibility_timer: Timer,
    pub bullet_damage_bonus: i32,
    pub projectile_speed_multiplier: f32,
    pub projectile_piercing: u32,    
    pub xp_gain_multiplier: f32,     
    pub pickup_radius_multiplier: f32,
    pub additional_projectiles: u32,
    pub max_health: i32, 
    pub health_regen_rate: f32,
}

impl Player {
    pub fn experience_to_next_level(&self) -> u32 {
        if self.level == 0 { return 0; } // Should start at level 1
        if (self.level as usize -1) < XP_FOR_LEVEL.len() {
            XP_FOR_LEVEL[self.level as usize - 1]
        } else {
            XP_FOR_LEVEL.last().unwrap_or(&2500) + (self.level - XP_FOR_LEVEL.len() as u32) * 500
        }
    }

    pub fn add_experience(
        &mut self,
        amount: u32,
        next_state_value: &mut NextState<AppState>,
        sound_event_writer: &mut EventWriter<PlaySoundEvent>, 
    ) {
        let actual_xp_gained = (amount as f32 * self.xp_gain_multiplier).round() as u32;
        self.current_level_xp += actual_xp_gained;
        self.experience += actual_xp_gained;

        while self.current_level_xp >= self.experience_to_next_level() && self.level > 0 {
            let needed = self.experience_to_next_level();
            self.current_level_xp -= needed;
            self.level += 1;
            sound_event_writer.send(PlaySoundEvent(SoundEffect::LevelUp)); 
            next_state_value.set(AppState::LevelUp);
            // If multiple level ups occur from one XP gain, only one LevelUp state transition is triggered.
            // The UI will show the new (highest) level. Subsequent XP gain will check against the new threshold.
            if next_state_value.0 == Some(AppState::LevelUp) {
                break; 
            }
        }
    }

    pub fn get_effective_pickup_radius(&self) -> f32 {
        BASE_PICKUP_RADIUS * self.pickup_radius_multiplier
    }
}

fn should_despawn_player(next_state: Res<NextState<AppState>>) -> bool {
    match next_state.0 {
        Some(AppState::GameOver) | Some(AppState::MainMenu) => true,
        _ => false,
    }
}

fn no_player_exists(player_query: Query<(), With<Player>>) -> bool {
    player_query.is_empty()
}


impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), 
                spawn_player.run_if(no_player_exists)
            )
            .add_systems(Update, (
                player_movement,
                player_aiming,
                player_shooting_system,
                player_health_regeneration_system,
                player_enemy_collision_system,
                player_invincibility_system,
                check_player_death_system,
            ).chain().run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), 
                despawn_player.run_if(should_despawn_player)
            );
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/player_ship.png"),
            sprite: Sprite {
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Player {
            speed: 250.0,
            experience: 0,
            current_level_xp: 0,
            level: 1,
            aim_direction: Vec2::X,
            invincibility_timer: Timer::from_seconds(1.0, TimerMode::Once),
            bullet_damage_bonus: 0,
            projectile_speed_multiplier: 1.0,
            projectile_piercing: 0, 
            xp_gain_multiplier: 1.0, 
            pickup_radius_multiplier: 1.0,
            additional_projectiles: 0,
            max_health: INITIAL_PLAYER_MAX_HEALTH,
            health_regen_rate: 0.0,
        },
        ComponentHealth(INITIAL_PLAYER_MAX_HEALTH),
        Velocity(Vec2::ZERO),
        BasicWeapon {
            fire_rate: Timer::from_seconds(0.5, TimerMode::Repeating),
        },
        AoeAuraWeapon::default(), 
        OrbitingProjectileWeapon::default(), // Add new weapon component
        Name::new("Player"),
    ));
}

fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    if let Ok(player_entity) = player_query.get_single() {
        commands.entity(player_entity).despawn_recursive();
    }
}

fn player_health_regeneration_system(
    time: Res<Time>,
    mut query: Query<(&Player, &mut ComponentHealth)>, 
) {
    for (player_stats, mut current_health) in query.iter_mut() {
        if player_stats.health_regen_rate > 0.0 && current_health.0 > 0 && current_health.0 < player_stats.max_health {
            let regen_amount = player_stats.health_regen_rate * time.delta_seconds();
            current_health.0 = (current_health.0 as f32 + regen_amount).round() as i32;
            current_health.0 = current_health.0.min(player_stats.max_health);
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform, &mut Velocity)>,
    time: Res<Time>,
) {
    for (player, mut transform, mut velocity) in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }
        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; }

        velocity.0 = if direction != Vec2::ZERO { direction.normalize() * player.speed } else { Vec2::ZERO };

        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn player_aiming(
    mut player_query: Query<(&mut Player, &Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok((mut player, player_transform)) = player_query.get_single_mut() else { return };
    let Ok(primary_window) = window_query.get_single() else { return };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };

    if let Some(cursor_position) = primary_window.cursor_position() {
        if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            let direction_to_mouse = (world_position - player_transform.translation.truncate()).normalize_or_zero();
            if direction_to_mouse != Vec2::ZERO {
                player.aim_direction = direction_to_mouse;
            }
        }
    }
}

fn player_shooting_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(&Transform, &Player, &mut BasicWeapon)>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (player_transform, player_stats, mut weapon) in query.iter_mut() {
        weapon.fire_rate.tick(time.delta());
        if weapon.fire_rate.just_finished() {
            if player_stats.aim_direction != Vec2::ZERO {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerShoot));
                let current_damage = BASE_BULLET_DAMAGE + player_stats.bullet_damage_bonus;
                let current_speed = BASE_BULLET_SPEED * player_stats.projectile_speed_multiplier;
                let current_piercing = player_stats.projectile_piercing;
                let total_projectiles = 1 + player_stats.additional_projectiles;

                let base_angle = player_stats.aim_direction.to_angle();

                for i in 0..total_projectiles {
                    let angle_offset_rad = if total_projectiles > 1 {
                        let total_spread_angle_rad = (total_projectiles as f32 - 1.0) * PROJECTILE_SPREAD_ANGLE_DEGREES.to_radians();
                        let start_angle_rad = base_angle - total_spread_angle_rad / 2.0;
                        start_angle_rad + (i as f32 * PROJECTILE_SPREAD_ANGLE_DEGREES.to_radians())
                    } else {
                        base_angle
                    };
                    
                    let projectile_direction = Vec2::from_angle(angle_offset_rad);

                    spawn_bullet(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,
                        projectile_direction,
                        current_damage,
                        current_speed,
                        current_piercing,
                    );
                }
            }
        }
    }
}

fn player_enemy_collision_system(
    mut player_query: Query<(&Transform, &mut ComponentHealth, &mut Player)>,
    enemy_query: Query<(&Transform, &Enemy)>, 
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    let Ok((player_transform, mut player_health, mut player_component)) = player_query.get_single_mut() else { return };

    if !player_component.invincibility_timer.finished() {
        return; 
    }

    for (enemy_transform, enemy_stats) in enemy_query.iter() {
        let distance = player_transform.translation.truncate().distance(enemy_transform.translation.truncate());
        let player_radius = PLAYER_SIZE.x / 2.0;
        let enemy_radius = enemy_stats.size.x / 2.0;

        if distance < player_radius + enemy_radius {
            if player_component.invincibility_timer.finished() { 
                sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerHit));
                player_health.0 -= enemy_stats.damage_on_collision;
                player_component.invincibility_timer.reset();
            }
        }
    }
}

fn player_invincibility_system(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Sprite, &ComponentHealth)>, 
) {
    for (mut player, mut sprite, health) in query.iter_mut() {
        if health.0 <= 0 { 
             if sprite.color.a() != 1.0 { sprite.color.set_a(1.0); }
            continue;
        }

        if !player.invincibility_timer.finished() {
            player.invincibility_timer.tick(time.delta());
            let alpha = (time.elapsed_seconds() * 20.0).sin() / 2.0 + 0.7; 
             sprite.color.set_a(alpha.clamp(0.3, 1.0) as f32); 
        } else {
            if sprite.color.a() != 1.0 { 
                 sprite.color.set_a(1.0);
            }
        }
    }
}

fn check_player_death_system(
    player_query: Query<&ComponentHealth, With<Player>>,
    mut app_state_next: ResMut<NextState<AppState>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
    current_app_state: Res<State<AppState>>,
) {
    if let Ok(player_health) = player_query.get_single() {
        if player_health.0 <= 0 && *current_app_state.get() == AppState::InGame {
            sound_event_writer.send(PlaySoundEvent(SoundEffect::GameOver));
            app_state_next.set(AppState::GameOver);
        }
    }
}