use bevy::prelude::*;
use rand::Rng;
use crate::{
    components::{Velocity, Health, Damage, Lifetime}, 
    player::Player,
    game::{AppState, GameState},
    audio::{PlaySoundEvent, SoundEffect}, 
};

pub const ENEMY_GRUNT_SIZE: Vec2 = Vec2::new(40.0, 40.0);
pub const ENEMY_SPITTER_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const ENEMY_TANK_SIZE: Vec2 = Vec2::new(55.0, 55.0);

#[derive(Resource)]
pub struct MaxEnemies(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    Grunt, 
    Spitter,
    Tank,
}

pub struct EnemyStats {
    pub enemy_type: EnemyType,
    pub health: i32,
    pub damage_on_collision: i32,
    pub speed: f32,
    pub size: Vec2,
    pub sprite_path: &'static str,
    pub spitter_range: Option<f32>,
    pub spitter_fire_rate: Option<f32>,
    pub spitter_projectile_speed: Option<f32>,
    pub spitter_projectile_damage: Option<i32>,
}

impl EnemyStats {
    fn get_for_type(enemy_type: EnemyType, wave_multiplier: f32) -> Self {
        match enemy_type {
            EnemyType::Grunt => EnemyStats {
                enemy_type,
                health: (20.0 * wave_multiplier).max(1.0) as i32,
                damage_on_collision: 10,
                speed: 100.0 + 20.0 * (wave_multiplier - 1.0).max(0.0),
                size: ENEMY_GRUNT_SIZE, 
                sprite_path: "sprites/enemy_grunt.png",
                spitter_range: None,
                spitter_fire_rate: None,
                spitter_projectile_speed: None,
                spitter_projectile_damage: None,
            },
            EnemyType::Spitter => EnemyStats {
                enemy_type,
                health: (15.0 * wave_multiplier).max(1.0) as i32,
                damage_on_collision: 5,
                speed: 80.0 + 15.0 * (wave_multiplier - 1.0).max(0.0),
                size: ENEMY_SPITTER_SIZE,
                sprite_path: "sprites/enemy_spitter.png",
                spitter_range: Some(300.0),
                spitter_fire_rate: Some(2.5), 
                spitter_projectile_speed: Some(250.0),
                spitter_projectile_damage: Some(8),
            },
            EnemyType::Tank => EnemyStats {
                enemy_type,
                health: (50.0 * wave_multiplier * 1.5).max(1.0) as i32,
                damage_on_collision: 15,
                speed: 60.0 + 10.0 * (wave_multiplier - 1.0).max(0.0), 
                size: ENEMY_TANK_SIZE, 
                sprite_path: "sprites/enemy_tank.png", 
                spitter_range: None,
                spitter_fire_rate: None,
                spitter_projectile_speed: None,
                spitter_projectile_damage: None,
            },
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType, 
    pub size: Vec2,
    pub damage_on_collision: i32,
    pub speed: f32, 
}

#[derive(Component)]
pub struct SpitterBehavior {
    pub shooting_range: f32,
    pub fire_timer: Timer,
    pub projectile_speed: f32,
    pub projectile_damage: i32,
}

#[derive(Component)]
pub struct EnemyProjectile; 

// Temporarily increase size and make color very obvious for debugging
const SPITTER_PROJECTILE_SIZE_DEBUG: Vec2 = Vec2::new(25.0, 25.0); // Larger for debug
const SPITTER_PROJECTILE_COLOR_DEBUG: Color = Color::FUCHSIA; // Bright color
const SPITTER_PROJECTILE_LIFETIME: f32 = 3.0;
const ENEMY_PROJECTILE_Z: f32 = 0.7; // Ensure it's above spitter (0.5) and other enemies

fn spawn_spitter_projectile(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut position: Vec3,
    direction: Vec2,
    speed: f32,
    damage: i32,
) {
    // info!("Attempting to spawn spitter projectile at {:?} with direction {:?}", position, direction); // Log
    position.z = ENEMY_PROJECTILE_Z; 
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/enemy_projectile.png"), 
            sprite: Sprite { 
                custom_size: Some(SPITTER_PROJECTILE_SIZE_DEBUG), // Use debug size
                color: SPITTER_PROJECTILE_COLOR_DEBUG, // Use debug color
                ..default()
            },
            visibility: Visibility::Visible, // Explicitly set visibility
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        EnemyProjectile,
        Velocity(direction * speed),
        Damage(damage),
        Lifetime { timer: Timer::from_seconds(SPITTER_PROJECTILE_LIFETIME, TimerMode::Once)},
        Name::new("SpitterProjectile"),
    ));
}


#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

pub struct EnemyPlugin;

fn should_despawn_all_entities_on_session_end(next_state: Res<NextState<AppState>>) -> bool {
    match next_state.0 {
        Some(AppState::MainMenu) | Some(AppState::GameOver) => true,
        _ => false,
    }
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            (
                enemy_spawn_system,
                enemy_movement_system, 
                spitter_shooting_logic, 
                enemy_projectile_collision_system, 
                enemy_projectile_lifetime_system,
            ).chain()
            .run_if(in_state(AppState::InGame))
        )
        .add_systems(PostUpdate, 
            update_enemy_count_system_in_game_state.run_if(in_state(AppState::InGame))
        ) 
        .add_systems(OnExit(AppState::InGame), 
            despawn_all_enemies.run_if(should_despawn_all_entities_on_session_end)
        );
    }
}

pub fn despawn_all_enemies(mut commands: Commands, enemy_query: Query<Entity, With<Enemy>>) {
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}


fn enemy_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(), With<Enemy>>,
    max_enemies: Res<MaxEnemies>,
    game_state: Res<GameState>,
) {
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() || enemy_query.iter().count() >= max_enemies.0 as usize {
        return;
    }

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
        1..=2 => EnemyType::Grunt, // Make sure Spitters can spawn early for testing if needed
        3..=5 => { 
            if rng.gen_bool(0.6) { EnemyType::Grunt } 
            else if rng.gen_bool(0.5) { EnemyType::Spitter } // Increase chance of Spitter for testing
            else { EnemyType::Tank }
        }
        _ => { 
            let enemy_type_roll = rng.gen_range(0..100);
            if enemy_type_roll < 40 { EnemyType::Grunt }
            else if enemy_type_roll < 80 { EnemyType::Spitter } // Higher chance for testing
            else { EnemyType::Tank }
        }
    };
    
    let stats = EnemyStats::get_for_type(chosen_type, wave_multiplier);
    let mut enemy_entity_commands = commands.spawn((
        SpriteBundle {
            texture: asset_server.load(stats.sprite_path),
            sprite: Sprite { custom_size: Some(stats.size), ..default() },
            transform: Transform::from_translation(final_spawn_pos),
            ..default()
        },
        Enemy {
            enemy_type: stats.enemy_type,
            size: stats.size,
            damage_on_collision: stats.damage_on_collision,
            speed: stats.speed,
        },
        Health(stats.health),
        Velocity(Vec2::ZERO),
        Name::new(format!("{:?}", stats.enemy_type)),
    ));

    if chosen_type == EnemyType::Spitter {
        enemy_entity_commands.insert(SpitterBehavior {
            shooting_range: stats.spitter_range.unwrap_or(300.0),
            fire_timer: Timer::from_seconds(stats.spitter_fire_rate.unwrap_or(1.5), TimerMode::Repeating), // Faster for testing
            projectile_speed: stats.spitter_projectile_speed.unwrap_or(250.0),
            projectile_damage: stats.spitter_projectile_damage.unwrap_or(8),
        });
    }
}

fn enemy_movement_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Enemy, Option<&SpitterBehavior>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else { return; };
    let player_pos = player_transform.translation.truncate();

    for (mut transform, mut velocity, enemy_data, spitter_opt) in query.iter_mut() {
        let enemy_pos = transform.translation.truncate();
        
        let mut should_move = true;
        if let Some(spitter) = spitter_opt {
            if enemy_pos.distance(player_pos) <= spitter.shooting_range {
                should_move = false; 
            }
        }

        if should_move {
            let direction_to_player = (player_pos - enemy_pos).normalize_or_zero();
            velocity.0 = direction_to_player * enemy_data.speed;
            transform.translation.x += velocity.0.x * time.delta_seconds();
            transform.translation.y += velocity.0.y * time.delta_seconds();
            if direction_to_player != Vec2::ZERO {
                 transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x));
            }
        }
    }
}

fn spitter_shooting_logic(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut spitter_query: Query<(&mut Transform, &mut Velocity, &mut SpitterBehavior, &GlobalTransform), (With<Enemy>, With<SpitterBehavior>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    let Ok(player_transform) = player_query.get_single() else { return; };
    let player_position = player_transform.translation.truncate();

    for (mut transform, mut velocity, mut spitter, spitter_gtransform) in spitter_query.iter_mut() {
        let spitter_position = spitter_gtransform.translation().truncate();
        let distance_to_player = player_position.distance(spitter_position);
        
        if distance_to_player <= spitter.shooting_range {
            // info!("Spitter in range, stopping and attempting to shoot."); // Log
            velocity.0 = Vec2::ZERO; 
            let direction_to_player = (player_position - spitter_position).normalize_or_zero();
            if direction_to_player != Vec2::ZERO {
                transform.rotation = Quat::from_rotation_z(direction_to_player.y.atan2(direction_to_player.x));
            }

            spitter.fire_timer.tick(time.delta());
            if spitter.fire_timer.just_finished() {
                // info!("Spitter firing!"); // Log
                sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyShoot));
                spawn_spitter_projectile(
                    &mut commands,
                    &asset_server,
                    spitter_gtransform.translation(), 
                    direction_to_player,
                    spitter.projectile_speed,
                    spitter.projectile_damage,
                );
            }
        }
    }
}

fn enemy_projectile_collision_system(
    mut commands: Commands,
    projectile_query: Query<(Entity, &GlobalTransform, &Damage), With<EnemyProjectile>>,
    mut player_query: Query<(&GlobalTransform, &mut Health, &mut Player), With<Player>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_gtransform, mut player_health, mut player_component)) = player_query.get_single_mut() {
        for (projectile_entity, projectile_gtransform, projectile_damage) in projectile_query.iter() {
            let distance = projectile_gtransform.translation().truncate()
                .distance(player_gtransform.translation().truncate());
            
            let projectile_radius = SPITTER_PROJECTILE_SIZE_DEBUG.x / 2.0; // Use debug size for collision too if it's larger
            let player_radius = crate::player::PLAYER_SIZE.x / 2.0;

            if distance < projectile_radius + player_radius {
                if player_component.invincibility_timer.finished() { 
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::PlayerHit));
                    player_health.0 -= projectile_damage.0;
                    player_component.invincibility_timer.reset();
                }
                commands.entity(projectile_entity).despawn_recursive(); 
            }
        }
    }
}

fn enemy_projectile_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime), With<EnemyProjectile>>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_enemy_count_system_in_game_state(
    mut game_state: ResMut<crate::game::GameState>,
    enemy_query: Query<(), With<Enemy>>,
) {
    game_state.enemy_count = enemy_query.iter().count() as u32;
}