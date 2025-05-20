use bevy::prelude::*;
use crate::{
    survivor::Survivor, // Changed
    horror::Horror,     // Changed
    game::AppState, 
};

const LEVEL_UP_WAVE_DURATION_SECONDS: f32 = 0.75; 
const LEVEL_UP_WAVE_MAX_RADIUS: f32 = 1000.0; 

pub struct LevelEventEffectsPlugin;

impl Plugin for LevelEventEffectsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::LevelUp), spawn_level_up_wave_effect)
            .add_systems(Update, 
                process_level_up_wave_effect
                    .run_if(in_state(AppState::LevelUp))
            );
    }
}

#[derive(Component)]
pub struct LevelUpWaveEffect {
    pub origin: Vec2,
    pub start_time: f32, 
    pub current_radius: f32,
}

fn spawn_level_up_wave_effect(
    mut commands: Commands,
    player_query: Query<&Transform, With<Survivor>>, // Changed
    time: Res<Time>, 
    asset_server: Res<AssetServer>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_position = player_transform.translation.truncate();
        
        commands.spawn((
            LevelUpWaveEffect {
                origin: player_position,
                start_time: time.elapsed_seconds(),
                current_radius: 0.0,
            },
            SpriteBundle {
                texture: asset_server.load("sprites/revelation_wave_placeholder.png"), // Changed
                sprite: Sprite {
                    custom_size: Some(Vec2::new(1.0, 1.0)), 
                    color: Color::rgba(0.6, 0.6, 1.0, 0.6),
                    ..default()
                },
                transform: Transform::from_translation(player_position.extend(1.5)), 
                visibility: Visibility::Visible,
                ..default()
            },
            Name::new("RevelationWave"), // Changed
        ));
    }
}

fn process_level_up_wave_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_query: Query<(Entity, &mut LevelUpWaveEffect, &mut Transform, &mut Sprite)>,
    horror_query: Query<(Entity, &GlobalTransform), With<Horror>>, // Changed enemy_query to horror_query and With<Enemy> to With<Horror>
) {
    for (wave_entity, mut wave, mut wave_transform, mut wave_sprite) in wave_query.iter_mut() {
        let time_since_spawn = time.elapsed_seconds() - wave.start_time;
        let progress = (time_since_spawn / LEVEL_UP_WAVE_DURATION_SECONDS).clamp(0.0, 1.0);

        wave.current_radius = LEVEL_UP_WAVE_MAX_RADIUS * progress;
        
        let diameter = wave.current_radius * 2.0;
        wave_transform.scale = Vec3::splat(diameter);
        
        wave_sprite.color.set_a(0.6 * (1.0 - progress * progress));

        if progress > 0.0 && progress < 1.0 { 
            let mut _horrors_cleared_this_frame = 0; // Renamed
            for (horror_entity, horror_gtransform) in horror_query.iter() { // Changed
                let horror_position = horror_gtransform.translation().truncate(); // Changed
                if horror_position.distance(wave.origin) < wave.current_radius { // Changed
                    commands.entity(horror_entity).despawn_recursive(); // Changed
                    _horrors_cleared_this_frame += 1; // Renamed
                }
            }
        }

        if progress >= 1.0 {
            commands.entity(wave_entity).despawn_recursive();
        }
    }
}