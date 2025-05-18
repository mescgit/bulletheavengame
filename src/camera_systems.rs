use bevy::prelude::*;
use crate::player::Player;
use crate::game::AppState;

const CAMERA_LERP_FACTOR: f32 = 0.05; // Adjust for more or less "softness" (lower is softer)

#[derive(Component)]
pub struct MainCamera; // Marker component for the main game camera

pub struct CameraSystemsPlugin;

impl Plugin for CameraSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            soft_camera_follow_system.run_if(in_state(AppState::InGame))
        );
    }
}

fn soft_camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let target_position = player_transform.translation;
            
            // Interpolate camera position towards player position
            // Only interpolate X and Y, keep Z fixed unless desired.
            camera_transform.translation = camera_transform.translation.lerp(target_position, CAMERA_LERP_FACTOR);
            // Ensure camera Z remains constant if it was set specifically
            // camera_transform.translation.z = desired_camera_z_value; // e.g. 10.0 or what was set at spawn
        }
    }
}