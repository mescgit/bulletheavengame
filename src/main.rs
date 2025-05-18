use bevy::prelude::*;

mod player;
mod components;
mod enemy;
mod bullet;
mod game;
mod experience;
mod upgrades;
mod level_event_effects;
mod weapons; 
mod visual_effects;
mod audio;
mod camera_systems;
mod background; 
mod debug_menu; // Ensure this line is present

use player::PlayerPlugin;
use enemy::EnemyPlugin;
use bullet::BulletPlugin;
use game::{GamePlugin, SCREEN_WIDTH, SCREEN_HEIGHT}; // GamePlugin adds DebugMenuPlugin
use level_event_effects::LevelEventEffectsPlugin;
use weapons::WeaponsPlugin;
use visual_effects::VisualEffectsPlugin;
use audio::GameAudioPlugin;
use camera_systems::{CameraSystemsPlugin, MainCamera};
use background::BackgroundPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmic Gardener VS (Eldritch WIP)".into(), // Updated title
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            GamePlugin, // GamePlugin adds DebugMenuPlugin internally
            PlayerPlugin,
            EnemyPlugin,
            BulletPlugin,
            LevelEventEffectsPlugin,
            WeaponsPlugin,
            VisualEffectsPlugin,
            GameAudioPlugin,
            CameraSystemsPlugin,
            BackgroundPlugin,
        ))
        .add_systems(Startup, setup_global_camera)
        .run();
}

fn setup_global_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation.z = 999.0; 
    commands.spawn((camera_bundle, MainCamera));
}