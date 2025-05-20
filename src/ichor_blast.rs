use bevy::prelude::*;
use rand::Rng; // For chance
use crate::{
    components::{Velocity, Damage, Lifetime, Health},
    visual_effects::spawn_damage_text,
    audio::{PlaySoundEvent, SoundEffect},
    skills::SkillProjectile,
    enemy::EnemyProjectile,
    survivor::Survivor, // Changed from player::Player
    items::{ItemLibrary, ItemEffect, ExplosionEffect}, // For item effects & explosion component
};

pub const ICHOR_BLAST_SIZE: Vec2 = Vec2::new(10.0, 10.0);
pub const BASE_FRAGMENT_SPEED: f32 = 600.0;
pub const BASE_FRAGMENT_DAMAGE: i32 = 10;
pub const FRAGMENT_LIFETIME_SECONDS: f32 = 2.0;

pub struct IchorBlastPlugin;

impl Plugin for IchorBlastPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                projectile_movement_system,
                ichor_blast_collision_system,
                ichor_blast_lifetime_system,
            ).chain());
    }
}

#[derive(Component)]
pub struct IchorBlast {
    pub piercing_left: u32,
}

pub fn spawn_ichor_blast( commands: &mut Commands, asset_server: &Res<AssetServer>, position: Vec3, direction: Vec2, damage: i32, speed: f32, piercing: u32,) {
    commands.spawn(( SpriteBundle { texture: asset_server.load("sprites/ichor_blast_placeholder.png"), sprite: Sprite { custom_size: Some(ICHOR_BLAST_SIZE), color: Color::rgb(0.7, 0.5, 1.0), ..default() }, transform: Transform::from_translation(position).with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))), ..default() }, IchorBlast { piercing_left: piercing }, Velocity(direction * speed), Damage(damage), Lifetime { timer: Timer::from_seconds(FRAGMENT_LIFETIME_SECONDS, TimerMode::Once) }, Name::new("IchorBlast"), ));
}

fn projectile_movement_system( mut query: Query<(&mut Transform, &Velocity), Or<(With<IchorBlast>, With<EnemyProjectile>, With<SkillProjectile>)>>, time: Res<Time>,) {
    for (mut transform, velocity) in query.iter_mut() { transform.translation.x += velocity.0.x * time.delta_seconds(); transform.translation.y += velocity.0.y * time.delta_seconds(); }
}

fn ichor_blast_lifetime_system( mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Lifetime), With<IchorBlast>>, ) {
    for (entity, mut lifetime) in query.iter_mut() { lifetime.timer.tick(time.delta()); if lifetime.timer.just_finished() { commands.entity(entity).despawn_recursive(); } }
}

fn ichor_blast_collision_system(
    mut commands: Commands,
    mut fragment_query: Query<(Entity, &GlobalTransform, &Damage, &mut IchorBlast)>,
    mut enemy_query: Query<(Entity, &GlobalTransform, &mut Health, &crate::enemy::Horror)>,
    player_query: Query<&Survivor>, // Changed from Query<&crate::player::Survivor>
    item_library: Res<ItemLibrary>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    let Ok(player) = player_query.get_single() else { return };

    for (fragment_entity, fragment_gtransform, fragment_damage, mut fragment_stats) in fragment_query.iter_mut() {
        for (enemy_entity, enemy_gtransform, mut enemy_health, enemy_data) in enemy_query.iter_mut() {
            let distance = fragment_gtransform.translation().truncate().distance(enemy_gtransform.translation().truncate());
            let fragment_radius = ICHOR_BLAST_SIZE.x / 2.0;
            let enemy_radius = enemy_data.size.x / 2.0;

            if distance < fragment_radius + enemy_radius {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::HorrorHit));
                enemy_health.0 -= fragment_damage.0;
                spawn_damage_text(&mut commands, &asset_server, enemy_gtransform.translation(), fragment_damage.0, &time);

                let mut rng = rand::thread_rng();
                for item_id in player.collected_item_ids.iter() {
                    if let Some(item_def) = item_library.get_item_definition(*item_id) {
                        for effect in &item_def.effects {
                            if let ItemEffect::OnIchorBlastHitExplode { chance, explosion_damage, explosion_radius, explosion_color } = effect {
                                if rng.gen_bool((*chance).into()) {
                                    commands.spawn((
                                        SpriteBundle {
                                            texture: asset_server.load("sprites/eldritch_nova_effect_placeholder.png"),
                                            sprite: Sprite {
                                                custom_size: Some(Vec2::splat(0.1)), 
                                                color: *explosion_color,
                                                ..default()
                                            },
                                            transform: Transform::from_translation(enemy_gtransform.translation().truncate().extend(0.3)),
                                            ..default()
                                        },
                                        ExplosionEffect {
                                            damage: *explosion_damage,
                                            radius_sq: explosion_radius.powi(2),
                                            timer: Timer::from_seconds(0.3, TimerMode::Once), 
                                            already_hit_entities: vec![enemy_entity], 
                                        },
                                        Name::new("ItemHitExplosion"),
                                    ));
                                }
                            }
                        }
                    }
                }

                if fragment_stats.piercing_left > 0 {
                    fragment_stats.piercing_left -= 1;
                } else {
                    commands.entity(fragment_entity).despawn_recursive();
                    break; 
                }
            }
        }
    }
}
