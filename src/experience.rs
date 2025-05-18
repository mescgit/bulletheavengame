use bevy::prelude::*;
use crate::{
    player::{Player, PLAYER_SIZE}, 
    components::Velocity,
    game::AppState,
    audio::{PlaySoundEvent, SoundEffect},
};

pub const EXP_ORB_SIZE: Vec2 = Vec2::new(10.0, 10.0);
pub const EXP_ORB_VALUE: u32 = 25;
const ORB_GRAVITATE_SPEED: f32 = 300.0;
const ORB_PICKUP_RADIUS_COLLISION: f32 = PLAYER_SIZE.x / 2.0 + EXP_ORB_SIZE.x / 2.0 - 5.0;


pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                orb_gravitation_and_movement_system,
                orb_collection_system,
            ).chain().run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct ExperienceOrb {
    pub value: u32,
}

pub fn spawn_experience_orb(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    value: u32,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/exp_orb.png"),
            sprite: Sprite {
                custom_size: Some(EXP_ORB_SIZE),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        ExperienceOrb { value },
        Velocity(Vec2::ZERO),
        Name::new("ExperienceOrb"),
    ));
}

fn orb_gravitation_and_movement_system(
    mut orb_query: Query<(&mut Transform, &mut Velocity), With<ExperienceOrb>>,
    player_query: Query<(&Transform, &Player), (With<Player>, Without<ExperienceOrb>)>,
    time: Res<Time>,
) {
    if let Ok((player_transform, player_stats)) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();
        let effective_gravitate_radius = player_stats.get_effective_pickup_radius();

        for (mut orb_transform, mut orb_velocity) in orb_query.iter_mut() {
            let orb_pos = orb_transform.translation.truncate();
            let distance_to_player = player_pos.distance(orb_pos);

            if distance_to_player < effective_gravitate_radius {
                let direction_to_player = (player_pos - orb_pos).normalize_or_zero();
                orb_velocity.0 = direction_to_player * ORB_GRAVITATE_SPEED;
            } else {
                 if orb_velocity.0 != Vec2::ZERO && distance_to_player > effective_gravitate_radius + 20.0 {
                     orb_velocity.0 = Vec2::ZERO;
                 }
            }
            
            orb_transform.translation.x += orb_velocity.0.x * time.delta_seconds();
            orb_transform.translation.y += orb_velocity.0.y * time.delta_seconds();
        }
    } else {
        for (mut orb_transform, mut orb_velocity) in orb_query.iter_mut() {
            if orb_velocity.0 != Vec2::ZERO {
                 orb_velocity.0 *= 0.9; 
                 if orb_velocity.0.length_squared() < 0.1 {
                     orb_velocity.0 = Vec2::ZERO;
                 }
            }
            orb_transform.translation.x += orb_velocity.0.x * time.delta_seconds();
            orb_transform.translation.y += orb_velocity.0.y * time.delta_seconds();
        }
    }
}

fn orb_collection_system(
    mut commands: Commands,
    orb_query: Query<(Entity, &Transform, &ExperienceOrb)>,
    mut player_query: Query<(&Transform, &mut Player), With<Player>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_transform, mut player_stats)) = player_query.get_single_mut() {
        let player_pos = player_transform.translation.truncate();
        for (orb_entity, orb_transform, orb_data) in orb_query.iter() {
            let orb_pos = orb_transform.translation.truncate();
            if player_pos.distance(orb_pos) < ORB_PICKUP_RADIUS_COLLISION { 
                commands.entity(orb_entity).despawn();
                sound_event_writer.send(PlaySoundEvent(SoundEffect::XpCollect));
                player_stats.add_experience(orb_data.value, &mut next_app_state, &mut sound_event_writer);
            }
        }
    }
}