use bevy::prelude::*;

mod player;
mod components;
mod enemy;
mod thought_fragment;
mod game;
mod experience;
mod upgrades;
mod level_event_effects;
mod weapons;
mod visual_effects;
mod audio;
mod camera_systems;
mod background;
mod debug_menu;
mod skills;
mod items;
mod glyphs;

use player::PlayerPlugin;
use enemy::EnemyPlugin;
use thought_fragment::ThoughtFragmentPlugin;
use game::{GamePlugin, SCREEN_WIDTH, SCREEN_HEIGHT};
use level_event_effects::LevelEventEffectsPlugin;
use weapons::WeaponsPlugin;
use visual_effects::VisualEffectsPlugin;
use audio::GameAudioPlugin;
use camera_systems::{CameraSystemsPlugin, MainCamera};
use background::BackgroundPlugin;
use skills::SkillsPlugin;
use items::ItemsPlugin;
use glyphs::GlyphsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Eldritch Hero".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            GamePlugin, PlayerPlugin, EnemyPlugin, ThoughtFragmentPlugin,
            LevelEventEffectsPlugin, WeaponsPlugin, VisualEffectsPlugin,
            GameAudioPlugin, CameraSystemsPlugin, BackgroundPlugin,
            SkillsPlugin, ItemsPlugin, GlyphsPlugin,
        ))
        .add_systems(Startup, setup_global_camera)
        .run();
}

fn setup_global_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation.z = 999.0;
    commands.spawn((camera_bundle, MainCamera));
}