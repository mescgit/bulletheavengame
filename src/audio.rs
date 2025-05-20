use bevy::prelude::*;
use crate::game::AppState;

#[derive(Event)]
pub struct PlaySoundEvent(pub SoundEffect);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    RitualCast,
    HorrorHit,
    HorrorDeath,
    SurvivorHit,
    Revelation,
    SoulCollect,
    MadnessConsumes,
    OmenAccepted,
    HorrorProjectile, 
}

#[derive(Resource)]
pub struct GameAudioHandles {
    pub ritual_cast: Handle<AudioSource>,
    pub horror_hit: Handle<AudioSource>,
    pub horror_death: Handle<AudioSource>,
    pub survivor_hit: Handle<AudioSource>,
    pub revelation: Handle<AudioSource>,
    pub soul_collect: Handle<AudioSource>,
    pub madness_consumes: Handle<AudioSource>,
    pub omen_accepted: Handle<AudioSource>,
    pub horror_projectile: Handle<AudioSource>,
    pub background_music: Handle<AudioSource>,
}

#[derive(Component)]
struct BackgroundMusicController;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlaySoundEvent>()
            .add_systems(Startup, setup_audio_handles)
            .add_systems(Update, play_sound_system)
            .add_systems(OnEnter(AppState::InGame), start_background_music)
            .add_systems(OnExit(AppState::InGame), stop_background_music);
    }
}

fn setup_audio_handles(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAudioHandles {
        ritual_cast: asset_server.load("audio/ritual_cast_placeholder.ogg"),
        horror_hit: asset_server.load("audio/horror_hit_placeholder.ogg"),
        horror_death: asset_server.load("audio/horror_death_placeholder.ogg"),
        survivor_hit: asset_server.load("audio/survivor_hit_placeholder.ogg"),
        revelation: asset_server.load("audio/revelation_placeholder.ogg"),
        soul_collect: asset_server.load("audio/soul_collect_placeholder.ogg"),
        madness_consumes: asset_server.load("audio/madness_consumes_placeholder.ogg"),
        omen_accepted: asset_server.load("audio/omen_accepted_placeholder.ogg"),
        horror_projectile: asset_server.load("audio/horror_projectile_placeholder.ogg"), 
        background_music: asset_server.load("audio/cyclopean_ruins_ambience_placeholder.ogg"),
    });
}

fn play_sound_system(
    mut commands: Commands,
    mut sound_events: EventReader<PlaySoundEvent>,
    audio_handles: Res<GameAudioHandles>,
) {
    for event in sound_events.read() {
        let source = match event.0 {
            SoundEffect::RitualCast => audio_handles.ritual_cast.clone(),
            SoundEffect::HorrorHit => audio_handles.horror_hit.clone(),
            SoundEffect::HorrorDeath => audio_handles.horror_death.clone(),
            SoundEffect::SurvivorHit => audio_handles.survivor_hit.clone(),
            SoundEffect::Revelation => audio_handles.revelation.clone(),
            SoundEffect::SoulCollect => audio_handles.soul_collect.clone(),
            SoundEffect::MadnessConsumes => audio_handles.madness_consumes.clone(),
            SoundEffect::OmenAccepted => audio_handles.omen_accepted.clone(),
            SoundEffect::HorrorProjectile => audio_handles.horror_projectile.clone(),
        };
        commands.spawn(AudioBundle {
            source,
            settings: PlaybackSettings::DESPAWN, 
        });
    }
}

fn start_background_music(
    mut commands: Commands,
    audio_handles: Res<GameAudioHandles>,
    music_controller_query: Query<Entity, With<BackgroundMusicController>>, 
) {
    if !music_controller_query.is_empty() {
        return;
    }
    commands.spawn((
        AudioBundle {
            source: audio_handles.background_music.clone(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.3), 
                ..default()
            },
        },
        BackgroundMusicController,
    ));
}

fn stop_background_music(
    mut commands: Commands,
    music_controller_query: Query<Entity, With<BackgroundMusicController>>,
) {
    for entity in music_controller_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}