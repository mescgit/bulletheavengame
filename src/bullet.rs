use bevy::prelude::*;
use crate::{
    components::{Velocity, Damage, Lifetime, Health},
    // Import EnemyProjectile marker from enemy module to include it in movement system
    enemy::{Enemy, EnemyProjectile}, 
    game::GameState,
    experience::spawn_experience_orb, 
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
};


pub const BULLET_SIZE: Vec2 = Vec2::new(10.0, 10.0);
pub const BASE_BULLET_SPEED: f32 = 600.0;
pub const BASE_BULLET_DAMAGE: i32 = 10;
pub const BULLET_LIFETIME_SECONDS: f32 = 2.0;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                // Renamed bullet_movement to projectile_movement_system
                projectile_movement_system, 
                bullet_collision_system, // Player bullet collision with enemies
                bullet_lifetime_system,  // Player bullet lifetime
            ).chain());
        // Note: Enemy projectile collision with player and lifetime are in enemy.rs
    }
}

#[derive(Component)]
pub struct Bullet { // Marker for player's bullets
    pub piercing_left: u32,
}

#[derive(Component)]
pub struct BasicWeapon {
    pub fire_rate: Timer,
}

pub fn spawn_bullet(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    direction: Vec2,
    damage: i32,
    speed: f32,
    piercing: u32,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/seed_bullet.png"),
            sprite: Sprite { custom_size: Some(BULLET_SIZE), ..default() },
            transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        Bullet { piercing_left: piercing },
        Velocity(direction * speed),
        Damage(damage),
        Lifetime { timer: Timer::from_seconds(BULLET_LIFETIME_SECONDS, TimerMode::Once) },
        Name::new("SeedBullet"),
    ));
}

// This system now moves any entity with Velocity and either a Bullet or EnemyProjectile component.
fn projectile_movement_system(
    // Query for entities that are either player bullets or enemy projectiles and have Velocity
    mut query: Query<(&mut Transform, &Velocity), Or<(With<Bullet>, With<EnemyProjectile>)>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
        // We could also update rotation here if projectiles should face their movement direction
        // and aren't just set once at spawn. For simple projectiles, this is often not needed.
        // if velocity.0 != Vec2::ZERO {
        //    transform.rotation = Quat::from_rotation_z(velocity.0.y.atan2(velocity.0.x));
        // }
    }
}

fn bullet_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    // This system specifically targets player bullets for their lifetime
    mut query: Query<(Entity, &mut Lifetime), With<Bullet>>, 
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_collision_system(
    mut commands: Commands,
    // This system is for player bullets hitting enemies
    mut bullet_query: Query<(Entity, &GlobalTransform, &Damage, &mut Bullet)>, 
    mut enemy_query: Query<(Entity, &GlobalTransform, &mut Health, &Transform, &Enemy)>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (bullet_entity, bullet_gtransform, bullet_damage, mut bullet_stats) in bullet_query.iter_mut() {
        let mut already_hit_this_frame: Vec<Entity> = Vec::new();

        for (enemy_entity, enemy_gtransform, mut enemy_health, enemy_transform_local, enemy_data) in enemy_query.iter_mut() {
            if already_hit_this_frame.contains(&enemy_entity) {
                continue;
            }

            let distance = bullet_gtransform.translation().truncate().distance(enemy_gtransform.translation().truncate());
            
            let bullet_radius = BULLET_SIZE.x / 2.0;
            let enemy_radius = enemy_data.size.x / 2.0;

            if distance < bullet_radius + enemy_radius {
                let actual_damage_dealt = bullet_damage.0;
                enemy_health.0 -= actual_damage_dealt;
                already_hit_this_frame.push(enemy_entity);
                
                sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyHit));
                spawn_damage_text(
                    &mut commands, 
                    &asset_server, 
                    enemy_gtransform.translation(),
                    actual_damage_dealt,
                    &time
                );

                if enemy_health.0 <= 0 {
                    sound_event_writer.send(PlaySoundEvent(SoundEffect::EnemyDeath));
                    let enemy_pos = enemy_transform_local.translation;
                    commands.entity(enemy_entity).despawn_recursive();
                    game_state.score += 10; 
                    spawn_experience_orb(&mut commands, &asset_server, enemy_pos, crate::experience::EXP_ORB_VALUE);
                }

                if bullet_stats.piercing_left > 0 {
                    bullet_stats.piercing_left -= 1;
                } else {
                    commands.entity(bullet_entity).despawn_recursive();
                    break; 
                }
            }
        }
    }
}