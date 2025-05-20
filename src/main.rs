use bevy::prelude::*;

mod survivor; // Changed
mod components;
mod horror; // Changed
mod ichor_blast; // Changed
mod game;
mod echoing_soul; // Changed
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

use survivor::SurvivorPlugin; // Changed
use horror::HorrorPlugin; // Changed
use ichor_blast::IchorBlastPlugin; // Changed
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
// Remove 'use experience::ExperiencePlugin' if it exists, as it's handled by GamePlugin

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Echoes of the Abyss".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            GamePlugin, // GamePlugin adds EchoingSoulPlugin (formerly ExperiencePlugin)
            SurvivorPlugin, 
            HorrorPlugin, 
            IchorBlastPlugin,
            LevelEventEffectsPlugin, 
            WeaponsPlugin, 
            VisualEffectsPlugin,
            GameAudioPlugin, 
            CameraSystemsPlugin, 
            BackgroundPlugin,
            SkillsPlugin, 
            ItemsPlugin, 
            GlyphsPlugin,
        ))
        .add_systems(Startup, setup_global_camera)
        .run();
}

fn setup_global_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation.z = 999.0; // Ensure camera is on top
    commands.spawn((camera_bundle, MainCamera));
}