use bevy::prelude::*;
use crate::game::AppState;

#[derive(Event)]
pub struct PlaySoundEvent(pub SoundEffect);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    PlayerShoot,
    EnemyHit,
    EnemyDeath,
    PlayerHit,
    LevelUp,
    XpCollect,
    GameOver,
    UpgradeChosen,
    EnemyShoot, 
}

#[derive(Resource)]
pub struct GameAudioHandles {
    pub shoot: Handle<AudioSource>,
    pub enemy_hit: Handle<AudioSource>,
    pub enemy_death: Handle<AudioSource>,
    pub player_hit: Handle<AudioSource>,
    pub level_up: Handle<AudioSource>,
    pub xp_collect: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub upgrade_chosen: Handle<AudioSource>,
    pub enemy_shoot: Handle<AudioSource>, 
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
        shoot: asset_server.load("audio/shoot.ogg"),
        enemy_hit: asset_server.load("audio/enemy_hit.ogg"),
        enemy_death: asset_server.load("audio/enemy_death.ogg"),
        player_hit: asset_server.load("audio/player_hit.ogg"),
        level_up: asset_server.load("audio/level_up.ogg"),
        xp_collect: asset_server.load("audio/xp_collect.ogg"),
        game_over: asset_server.load("audio/game_over.ogg"),
        upgrade_chosen: asset_server.load("audio/upgrade_chosen.ogg"),
        enemy_shoot: asset_server.load("audio/enemy_shoot.ogg"), 
        background_music: asset_server.load("audio/background_music.ogg"),
    });
}

fn play_sound_system(
    mut commands: Commands,
    mut sound_events: EventReader<PlaySoundEvent>,
    audio_handles: Res<GameAudioHandles>,
) {
    for event in sound_events.read() {
        let source = match event.0 {
            SoundEffect::PlayerShoot => audio_handles.shoot.clone(),
            SoundEffect::EnemyHit => audio_handles.enemy_hit.clone(),
            SoundEffect::EnemyDeath => audio_handles.enemy_death.clone(),
            SoundEffect::PlayerHit => audio_handles.player_hit.clone(),
            SoundEffect::LevelUp => audio_handles.level_up.clone(),
            SoundEffect::XpCollect => audio_handles.xp_collect.clone(),
            SoundEffect::GameOver => audio_handles.game_over.clone(),
            SoundEffect::UpgradeChosen => audio_handles.upgrade_chosen.clone(),
            SoundEffect::EnemyShoot => audio_handles.enemy_shoot.clone(),
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