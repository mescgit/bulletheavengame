use bevy::prelude::*;
use crate::{
    survivor::{Survivor, SURVIVOR_SIZE}, // Updated
    components::Velocity,
    game::AppState,
    audio::{PlaySoundEvent, SoundEffect},
};

pub const ECHOING_SOUL_SIZE: Vec2 = Vec2::new(10.0, 10.0);
pub const ECHOING_SOUL_VALUE: u32 = 25; 
const SOUL_GRAVITATE_SPEED: f32 = 300.0;
// Updated to use SURVIVOR_SIZE
const SOUL_PICKUP_RADIUS_COLLISION: f32 = SURVIVOR_SIZE.x / 2.0 + ECHOING_SOUL_SIZE.x / 2.0 - 5.0; 


pub struct EchoingSoulPlugin; // Renamed

impl Plugin for EchoingSoulPlugin { // Renamed
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                echoing_soul_gravitation_and_movement_system,
                echoing_soul_collection_system,
            ).chain().run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct EchoingSoul {
    pub value: u32,
}

pub fn spawn_echoing_soul(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    value: u32,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/echoing_soul_orb_placeholder.png"),
            sprite: Sprite {
                custom_size: Some(ECHOING_SOUL_SIZE),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        EchoingSoul { value },
        Velocity(Vec2::ZERO),
        Name::new("EchoingSoul"),
    ));
}

fn echoing_soul_gravitation_and_movement_system(
    mut soul_query: Query<(&mut Transform, &mut Velocity), With<EchoingSoul>>,
    player_query: Query<(&Transform, &Survivor), (With<Survivor>, Without<EchoingSoul>)>,
    time: Res<Time>,
) {
    if let Ok((player_transform, player_stats)) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();
        let effective_gravitate_radius = player_stats.get_effective_pickup_radius();

        for (mut soul_transform, mut soul_velocity) in soul_query.iter_mut() {
            let soul_pos = soul_transform.translation.truncate();
            let distance_to_player = player_pos.distance(soul_pos);

            if distance_to_player < effective_gravitate_radius {
                let direction_to_player = (player_pos - soul_pos).normalize_or_zero();
                soul_velocity.0 = direction_to_player * SOUL_GRAVITATE_SPEED;
            } else {
                 if soul_velocity.0 != Vec2::ZERO && distance_to_player > effective_gravitate_radius + 20.0 {
                     soul_velocity.0 = Vec2::ZERO;
                 }
            }
            
            soul_transform.translation.x += soul_velocity.0.x * time.delta_seconds();
            soul_transform.translation.y += soul_velocity.0.y * time.delta_seconds();
        }
    } else {
        for (mut soul_transform, mut soul_velocity) in soul_query.iter_mut() {
            if soul_velocity.0 != Vec2::ZERO {
                 soul_velocity.0 *= 0.9; 
                 if soul_velocity.0.length_squared() < 0.1 {
                     soul_velocity.0 = Vec2::ZERO;
                 }
            }
            soul_transform.translation.x += soul_velocity.0.x * time.delta_seconds();
            soul_transform.translation.y += soul_velocity.0.y * time.delta_seconds();
        }
    }
}

fn echoing_soul_collection_system(
    mut commands: Commands,
    soul_query: Query<(Entity, &Transform, &EchoingSoul)>,
    mut player_query: Query<(&Transform, &mut Survivor), With<Survivor>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    if let Ok((player_transform, mut player_stats)) = player_query.get_single_mut() {
        let player_pos = player_transform.translation.truncate();
        for (soul_entity, soul_transform, soul_data) in soul_query.iter() {
            let soul_pos = soul_transform.translation.truncate();
            if player_pos.distance(soul_pos) < SOUL_PICKUP_RADIUS_COLLISION { 
                commands.entity(soul_entity).despawn();
                sound_event_writer.send(PlaySoundEvent(SoundEffect::SoulCollect));
                player_stats.add_experience(soul_data.value, &mut next_app_state, &mut sound_event_writer);
            }
        }
    }
}
