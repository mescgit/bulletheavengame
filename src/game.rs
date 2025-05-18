use bevy::prelude::*;
use crate::{
    enemy::{EnemySpawnTimer, MaxEnemies},
    experience::{ExperienceOrb, ExperiencePlugin},
    bullet::{Bullet, BasicWeapon},
    player::Player,
    components::Health,
    upgrades::{UpgradePlugin, UpgradePool, OfferedUpgrades, UpgradeCard, UpgradeType},
    weapons::{AoeAuraWeapon, OrbitingProjectileWeapon},
    audio::{PlaySoundEvent, SoundEffect},
    debug_menu::DebugMenuPlugin, // This import should now work
};

pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;

const INITIAL_MAX_ENEMIES: u32 = 20;
const INITIAL_SPAWN_INTERVAL_SECONDS: f32 = 2.0;
const DIFFICULTY_INCREASE_INTERVAL_SECONDS: f32 = 30.0;
const MAX_ENEMIES_INCREMENT: u32 = 10;
const SPAWN_INTERVAL_DECREMENT_FACTOR: f32 = 0.9;
const MIN_SPAWN_INTERVAL_SECONDS: f32 = 0.3;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    LevelUp,
    GameOver,
    DebugUpgradeMenu, 
}

#[derive(Resource)]
pub struct GameConfig {
    pub width: f32,
    pub height: f32,
    pub spawn_area_padding: f32, 
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            spawn_area_padding: 50.0,
        }
    }
}

pub struct GamePlugin;

#[derive(Resource, Default)]
pub struct GameState {
    pub score: u32,
    pub wave_number: u32,
    pub enemy_count: u32,
    pub game_over_timer: Timer, 
    pub game_timer: Timer,
    pub difficulty_timer: Timer,
}

#[derive(Event)]
pub struct UpgradeChosenEvent(pub UpgradeCard);

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
struct LevelUpUI;

#[derive(Component)]
struct UpgradeButton(UpgradeCard);

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct InGameUI;

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct XPText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct WaveText;

fn reset_for_new_game_session(
    mut game_state: ResMut<GameState>, // Binding `game_state` itself doesn't need to be mut here
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut max_enemies: ResMut<MaxEnemies>,
    mut player_query: Query<(&mut Player, &mut Health, &mut BasicWeapon, &mut AoeAuraWeapon, &mut OrbitingProjectileWeapon)>,
) {
    // ResMut allows interior mutability, so we can directly modify fields.
    game_state.score = 0; 
    game_state.wave_number = 1;
    game_state.enemy_count = 0;
    game_state.game_timer = Timer::from_seconds(3600.0, TimerMode::Once);
    game_state.game_timer.reset();
    game_state.game_timer.unpause();
    game_state.difficulty_timer = Timer::from_seconds(DIFFICULTY_INCREASE_INTERVAL_SECONDS, TimerMode::Repeating);
    game_state.difficulty_timer.reset();
    
    enemy_spawn_timer.timer.set_duration(std::time::Duration::from_secs_f32(INITIAL_SPAWN_INTERVAL_SECONDS));
    enemy_spawn_timer.timer.reset();
    max_enemies.0 = INITIAL_MAX_ENEMIES;

    if let Ok((mut player, mut health, mut basic_weapon, mut aoe_aura, mut orbiting_weapon)) = player_query.get_single_mut() {
        player.experience = 0;
        player.current_level_xp = 0;
        player.level = 1;
        player.bullet_damage_bonus = 0;
        player.projectile_speed_multiplier = 1.0;
        player.projectile_piercing = 0;
        player.xp_gain_multiplier = 1.0;
        player.pickup_radius_multiplier = 1.0;
        player.additional_projectiles = 0;
        player.max_health = crate::player::INITIAL_PLAYER_MAX_HEALTH;
        player.health_regen_rate = 0.0;
        health.0 = crate::player::INITIAL_PLAYER_MAX_HEALTH;
        basic_weapon.fire_rate = Timer::from_seconds(0.5, TimerMode::Repeating);
        *aoe_aura = AoeAuraWeapon::default();
        *orbiting_weapon = OrbitingProjectileWeapon::default();
    }
}

fn on_enter_ingame_state_actions(mut game_state: ResMut<GameState>) {
    if game_state.game_timer.paused() {
        game_state.game_timer.unpause();
    }
    if game_state.difficulty_timer.paused() {
        game_state.difficulty_timer.unpause();
    }
}

fn on_enter_pause_like_state_actions(mut game_state: ResMut<GameState>) { 
    if !game_state.game_timer.paused() {
        game_state.game_timer.pause();
    }
    if !game_state.difficulty_timer.paused() {
        game_state.difficulty_timer.pause();
    }
}


impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<UpgradeChosenEvent>()
            .add_plugins((UpgradePlugin, DebugMenuPlugin)) 
            .init_state::<AppState>()
            .init_resource::<GameConfig>()
            .init_resource::<GameState>()
            .insert_resource(EnemySpawnTimer {
                timer: Timer::from_seconds(INITIAL_SPAWN_INTERVAL_SECONDS, TimerMode::Repeating),
            })
            .insert_resource(MaxEnemies(INITIAL_MAX_ENEMIES))
            .add_plugins(ExperiencePlugin)

            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu_ui)
            .add_systems(Update, 
                main_menu_input_system
                    .run_if(in_state(AppState::MainMenu))
            )
            .add_systems(OnExit(AppState::MainMenu), despawn_ui_by_marker::<MainMenuUI>)

            .add_systems(OnEnter(AppState::InGame), (
                on_enter_ingame_state_actions,
                setup_ingame_ui,
            ))
            .add_systems(Update, (
                update_ingame_ui,
                update_game_timer,
                difficulty_scaling_system,
                global_debug_key_listener, 
            ).chain().run_if(in_state(AppState::InGame).or_else(in_state(AppState::DebugUpgradeMenu))))
            .add_systems(OnExit(AppState::InGame), (
                cleanup_session_entities, 
                despawn_ui_by_marker::<InGameUI>
            ))

            .add_systems(OnEnter(AppState::LevelUp), (setup_level_up_ui, on_enter_pause_like_state_actions))
            .add_systems(Update, handle_upgrade_choice_interaction.run_if(in_state(AppState::LevelUp)))
            .add_systems(Update, apply_chosen_upgrade.run_if(on_event::<UpgradeChosenEvent>()))
            .add_systems(OnExit(AppState::LevelUp), (despawn_ui_by_marker::<LevelUpUI>, on_enter_ingame_state_actions)) 

            .add_systems(OnEnter(AppState::DebugUpgradeMenu), on_enter_pause_like_state_actions)
            .add_systems(OnExit(AppState::DebugUpgradeMenu), on_enter_ingame_state_actions)


            .add_systems(OnEnter(AppState::GameOver), setup_game_over_ui)
            .add_systems(Update, 
                game_over_input_system
                    .run_if(in_state(AppState::GameOver))
            )
            .add_systems(OnExit(AppState::GameOver), despawn_ui_by_marker::<GameOverUI>);
    }
}

fn global_debug_key_listener(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        match current_app_state.get() {
            AppState::InGame => {
                next_app_state.set(AppState::DebugUpgradeMenu);
            }
            AppState::DebugUpgradeMenu => {
                next_app_state.set(AppState::InGame);
            }
            _ => {} 
        }
    }
}


fn despawn_ui_by_marker<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_main_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ..default()
        },
        MainMenuUI,
    )).with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                "Cosmic Gardener",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 70.0,
                    color: Color::WHITE,
                },
            ).with_text_justify(JustifyText::Center)
        );
        parent.spawn(
            TextBundle::from_section(
                "Press SPACE to Start",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                },
            ).with_text_justify(JustifyText::Center)
        );
    });
}

fn main_menu_input_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    game_state: ResMut<GameState>,
    enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    max_enemies: ResMut<MaxEnemies>,
    player_query: Query<(&mut Player, &mut Health, &mut BasicWeapon, &mut AoeAuraWeapon, &mut OrbitingProjectileWeapon)>, 
    player_entity_query: Query<Entity, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in player_entity_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        reset_for_new_game_session(game_state, enemy_spawn_timer, max_enemies, player_query);
        next_app_state.set(AppState::InGame);
    }
}

fn setup_ingame_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                position_type: PositionType::Absolute,
                ..default()
            },
            z_index: ZIndex::Global(1),
            ..default()
        },
        InGameUI,
    )).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
            ..default()
        }).with_children(|top_bar| {
            top_bar.spawn((TextBundle::from_section(
                "Health: 100",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::GREEN, },
            ), HealthText));
            top_bar.spawn((TextBundle::from_section(
                "Level: 1",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::CYAN, },
            ), LevelText));
            top_bar.spawn((TextBundle::from_section(
                "XP: 0/100",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::YELLOW, },
            ), XPText));
            top_bar.spawn((TextBundle::from_section(
                "Wave: 1",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::ORANGE_RED, },
            ), WaveText));
        });
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            ..default()
        }).with_children(|bottom_bar| {
            bottom_bar.spawn((TextBundle::from_section(
                "Score: 0",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::WHITE, },
            ), ScoreText));
            bottom_bar.spawn((TextBundle::from_section(
                "Time: 00:00",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 20.0, color: Color::WHITE, },
            ), TimerText));
        });
    });
}

fn update_game_timer(mut game_state: ResMut<GameState>, time: Res<Time>) {
    if !game_state.game_timer.paused() {
        game_state.game_timer.tick(time.delta());
    }
}

fn difficulty_scaling_system(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut max_enemies: ResMut<MaxEnemies>,
) {
    if game_state.difficulty_timer.paused() { return; }

    game_state.difficulty_timer.tick(time.delta());

    if game_state.difficulty_timer.just_finished() {
        game_state.wave_number += 1;
        max_enemies.0 = (INITIAL_MAX_ENEMIES + (game_state.wave_number -1) * MAX_ENEMIES_INCREMENT).min(200);

        let current_duration = enemy_spawn_timer.timer.duration().as_secs_f32();
        let new_duration = (current_duration * SPAWN_INTERVAL_DECREMENT_FACTOR).max(MIN_SPAWN_INTERVAL_SECONDS);
        enemy_spawn_timer.timer.set_duration(std::time::Duration::from_secs_f32(new_duration));
    }
}

fn update_ingame_ui(
    player_query: Query<(&Player, &Health)>,
    game_state: Res<GameState>,
    mut ui_texts: ParamSet<(
        Query<&mut Text, With<HealthText>>,
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<XPText>>,
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<TimerText>>,
        Query<&mut Text, With<WaveText>>,
    )>,
) {
    if let Ok((player_stats, player_health)) = player_query.get_single() {
        if let Ok(mut text) = ui_texts.p0().get_single_mut() {
            text.sections[0].value = format!("Health: {}/{}", player_health.0, player_stats.max_health);
            if player_health.0 < player_stats.max_health / 3 { text.sections[0].style.color = Color::RED; }
            else if player_health.0 < player_stats.max_health * 2 / 3 { text.sections[0].style.color = Color::YELLOW; }
            else { text.sections[0].style.color = Color::GREEN; }
        }
        if let Ok(mut text) = ui_texts.p1().get_single_mut() {
            text.sections[0].value = format!("Level: {}", player_stats.level);
        }
        if let Ok(mut text) = ui_texts.p2().get_single_mut() {
            text.sections[0].value = format!("XP: {}/{}", player_stats.current_level_xp, player_stats.experience_to_next_level());
        }
    } else {
        if let Ok(mut text) = ui_texts.p0().get_single_mut() { text.sections[0].value = "Health: --/--".to_string(); }
        if let Ok(mut text) = ui_texts.p1().get_single_mut() { text.sections[0].value = "Level: --".to_string(); }
        if let Ok(mut text) = ui_texts.p2().get_single_mut() { text.sections[0].value = "XP: --/--".to_string(); }
    }

    if let Ok(mut text) = ui_texts.p3().get_single_mut() {
        text.sections[0].value = format!("Score: {}", game_state.score);
    }
    if let Ok(mut text) = ui_texts.p4().get_single_mut() {
        let elapsed_seconds = game_state.game_timer.elapsed().as_secs();
        let minutes = elapsed_seconds / 60;
        let seconds = elapsed_seconds % 60;
        text.sections[0].value = format!("Time: {:02}:{:02}", minutes, seconds);
    }
    if let Ok(mut text) = ui_texts.p5().get_single_mut() {
        text.sections[0].value = format!("Wave: {}", game_state.wave_number);
    }
}

fn setup_level_up_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<&Player>,
    upgrade_pool: Res<UpgradePool>,
) {
    let player_level = if let Ok(player) = player_query.get_single() {
        player.level
    } else { 0 };

    let current_offered_upgrades = OfferedUpgrades { choices: upgrade_pool.get_random_upgrades(3) };

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column, 
                row_gap: Val::Px(30.0),
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.2, 0.9).into(),
            z_index: ZIndex::Global(10),
            ..default()
        },
        LevelUpUI,
        current_offered_upgrades.clone(), 
    )).with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                format!("Level Up! Now Level: {}", player_level),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::GOLD,
                },
            ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default()})
        );
        
        for (index, card) in current_offered_upgrades.choices.iter().enumerate() {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        height: Val::Px(120.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    border_color: BorderColor(Color::DARK_GRAY),
                    background_color: Color::GRAY.into(),
                    ..default()
                },
                UpgradeButton(card.clone()),
                Name::new(format!("Upgrade Button {}", index + 1)),
            )).with_children(|button_parent| {
                button_parent.spawn(TextBundle::from_section(
                    &card.name,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ).with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() }));
                button_parent.spawn(TextBundle::from_section(
                    &card.description,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 18.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ));
            });
        }
    });
}

fn handle_upgrade_choice_interaction(
    mut interaction_query: Query<
        (&Interaction, &UpgradeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut upgrade_chosen_event: EventWriter<UpgradeChosenEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    level_up_ui_query: Query<&OfferedUpgrades, With<LevelUpUI>>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (interaction, upgrade_button_data, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                sound_event_writer.send(PlaySoundEvent(SoundEffect::UpgradeChosen));
                upgrade_chosen_event.send(UpgradeChosenEvent(upgrade_button_data.0.clone()));
                next_app_state.set(AppState::InGame);
                return;
            }
            Interaction::Hovered => {
                *bg_color = Color::DARK_GREEN.into();
            }
            Interaction::None => {
                *bg_color = Color::GRAY.into();
            }
        }
    }

    if let Ok(offered) = level_up_ui_query.get_single() {
        let choice_made = if keyboard_input.just_pressed(KeyCode::Digit1) && offered.choices.len() > 0 {
            Some(offered.choices[0].clone())
        } else if keyboard_input.just_pressed(KeyCode::Digit2) && offered.choices.len() > 1 {
            Some(offered.choices[1].clone())
        } else if keyboard_input.just_pressed(KeyCode::Digit3) && offered.choices.len() > 2 {
            Some(offered.choices[2].clone())
        } else { None };

        if let Some(chosen_card) = choice_made {
            sound_event_writer.send(PlaySoundEvent(SoundEffect::UpgradeChosen));
            upgrade_chosen_event.send(UpgradeChosenEvent(chosen_card));
            next_app_state.set(AppState::InGame);
        }
    }
}

fn apply_chosen_upgrade(
    mut events: EventReader<UpgradeChosenEvent>,
    mut player_query: Query<(&mut Player, &mut BasicWeapon, &mut Health, &mut AoeAuraWeapon, &mut OrbitingProjectileWeapon)>,
) {
    for event in events.read() {
        let Ok((mut player_stats, mut basic_weapon_stats, mut health_stats, mut aoe_aura_weapon, mut orbiting_weapon)) = player_query.get_single_mut() else { continue };
        
        match &event.0.upgrade_type {
            UpgradeType::PlayerSpeed(percentage) => {
                player_stats.speed *= 1.0 + (*percentage as f32 / 100.0);
            }
            UpgradeType::MaxHealth(amount) => {
                player_stats.max_health += *amount;
                health_stats.0 += *amount; 
                health_stats.0 = health_stats.0.min(player_stats.max_health); 
            }
            UpgradeType::BulletDamage(bonus_amount) => {
                player_stats.bullet_damage_bonus += *bonus_amount;
            }
            UpgradeType::FireRate(percentage) => {
                let reduction_factor = *percentage as f32 / 100.0;
                let current_duration = basic_weapon_stats.fire_rate.duration().as_secs_f32();
                let new_duration = (current_duration * (1.0 - reduction_factor)).max(0.05);
                basic_weapon_stats.fire_rate.set_duration(std::time::Duration::from_secs_f32(new_duration));
            }
            UpgradeType::ProjectileSpeed(percentage_increase) => {
                player_stats.projectile_speed_multiplier *= 1.0 + (*percentage_increase as f32 / 100.0);
            }
            UpgradeType::ProjectilePiercing(amount) => {
                player_stats.projectile_piercing += *amount;
            }
            UpgradeType::XPGainMultiplier(percentage) => {
                player_stats.xp_gain_multiplier *= 1.0 + (*percentage as f32 / 100.0);
            }
            UpgradeType::PickupRadiusIncrease(percentage) => {
                player_stats.pickup_radius_multiplier *= 1.0 + (*percentage as f32 / 100.0);
            }
            UpgradeType::IncreaseProjectileCount(amount) => {
                player_stats.additional_projectiles += *amount;
            }
            UpgradeType::UnlockAoeAuraWeapon => {
                if !aoe_aura_weapon.is_active {
                    aoe_aura_weapon.is_active = true;
                } else { 
                    aoe_aura_weapon.base_damage_per_tick += 1;
                    aoe_aura_weapon.current_radius *= 1.1;
                }
            }
            UpgradeType::IncreaseAoeAuraRadius(percentage) => {
                if aoe_aura_weapon.is_active {
                    aoe_aura_weapon.current_radius *= 1.0 + (*percentage as f32 / 100.0);
                }
            }
            UpgradeType::IncreaseAoeAuraDamage(amount) => {
                 if aoe_aura_weapon.is_active {
                    aoe_aura_weapon.base_damage_per_tick += *amount;
                }
            }
            UpgradeType::DecreaseAoeAuraTickRate(percentage) => {
                if aoe_aura_weapon.is_active {
                    let reduction_factor = *percentage as f32 / 100.0;
                    let current_tick_duration = aoe_aura_weapon.damage_tick_timer.duration().as_secs_f32();
                    let new_tick_duration = (current_tick_duration * (1.0 - reduction_factor)).max(0.1);
                    aoe_aura_weapon.damage_tick_timer.set_duration(std::time::Duration::from_secs_f32(new_tick_duration));
                }
            }
            UpgradeType::HealthRegeneration(amount) => {
                player_stats.health_regen_rate += *amount;
            }
            UpgradeType::UnlockOrbitingSeeds => {
                if !orbiting_weapon.is_active {
                    orbiting_weapon.is_active = true;
                    orbiting_weapon.num_orbiters = orbiting_weapon.num_orbiters.max(2);
                } else { 
                    orbiting_weapon.num_orbiters += 1;
                    orbiting_weapon.damage_per_hit += 1;
                }
            }
            UpgradeType::IncreaseOrbiterCount(count) => {
                if orbiting_weapon.is_active {
                    orbiting_weapon.num_orbiters += *count;
                }
            }
            UpgradeType::IncreaseOrbiterDamage(damage) => {
                if orbiting_weapon.is_active {
                    orbiting_weapon.damage_per_hit += *damage;
                }
            }
            UpgradeType::IncreaseOrbiterRadius(radius_increase) => {
                if orbiting_weapon.is_active {
                    orbiting_weapon.orbit_radius += *radius_increase;
                }
            }
            UpgradeType::IncreaseOrbiterRotationSpeed(speed_increase) => {
                 if orbiting_weapon.is_active {
                    orbiting_weapon.rotation_speed += *speed_increase;
                }
            }
        }
    }
}

fn setup_game_over_ui(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        NodeBundle {
             style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            ..default()
        },
        GameOverUI,
    )).with_children(|parent| {
        parent.spawn(
            TextBundle::from_section(
                "Game Over!",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::RED,
                },
            ).with_text_justify(JustifyText::Center)
        );
        parent.spawn(
             TextBundle::from_section(
                format!("Score: {}", game_state.score),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
            ).with_text_justify(JustifyText::Center)
        );
        parent.spawn(
             TextBundle::from_section(
                "Press R to Restart",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgba(0.8,0.8,0.8,1.0),
                },
            ).with_text_justify(JustifyText::Center)
        );
    });
}

fn game_over_input_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    game_state: ResMut<GameState>, // Corrected: remove mut from binding
    enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    max_enemies: ResMut<MaxEnemies>,
    player_query: Query<(&mut Player, &mut Health, &mut BasicWeapon, &mut AoeAuraWeapon, &mut OrbitingProjectileWeapon)>,
    player_entity_query: Query<Entity, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for entity in player_entity_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        reset_for_new_game_session(game_state, enemy_spawn_timer, max_enemies, player_query);
        next_app_state.set(AppState::MainMenu);
    }
}

fn cleanup_session_entities(
    mut commands: Commands,
    bullets_query: Query<Entity, With<Bullet>>,
    orbs_query: Query<Entity, With<ExperienceOrb>>,
) {
    for entity in bullets_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in orbs_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}