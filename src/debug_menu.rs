use bevy::prelude::*;
use crate::{
    upgrades::{UpgradePool, UpgradeCard},
    game::{AppState, UpgradeChosenEvent},
    audio::{PlaySoundEvent, SoundEffect},
};

pub struct DebugMenuPlugin;

impl Plugin for DebugMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::DebugUpgradeMenu), setup_debug_menu_ui)
            .add_systems(Update, 
                debug_menu_button_interaction_system
                    .run_if(in_state(AppState::DebugUpgradeMenu))
            )
            // Note: The F12 key to close is handled by global_debug_key_listener in game.rs
            .add_systems(OnExit(AppState::DebugUpgradeMenu), despawn_debug_menu_ui);
    }
}

#[derive(Component)]
struct DebugMenuUIRoot;

#[derive(Component)]
struct DebugUpgradeButton(UpgradeCard);

const DEBUG_BUTTON_HEIGHT: Val = Val::Px(30.0);
const DEBUG_BUTTON_MARGIN: Val = Val::Px(5.0);
const DEBUG_TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const DEBUG_BUTTON_BG_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
const DEBUG_BUTTON_HOVER_BG_COLOR: Color = Color::rgb(0.35, 0.35, 0.35);
const DEBUG_BUTTON_PRESSED_BG_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);


fn setup_debug_menu_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    upgrade_pool: Res<UpgradePool>
) {
    info!("Setting up Debug Upgrade Menu UI");
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.85).into(),
            z_index: ZIndex::Global(50), // High Z-index to be on top
            ..default()
        },
        DebugMenuUIRoot,
        Name::new("DebugMenuUIRoot"),
    )).with_children(|parent| {
        // Container for scrollable list
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(50.0), // Take 50% of screen width
                min_width: Val::Px(400.0), // Minimum width
                max_width: Val::Px(700.0), // Maximum width
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            border_color: Color::DARK_GRAY.into(),
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        }).with_children(|list_container| {
            list_container.spawn(TextBundle::from_section(
                "DEBUG UPGRADE MENU (F12 to Toggle)",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 24.0,
                    color: Color::ORANGE_RED,
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(15.0)),
                align_self: AlignSelf::Center,
                ..default()
            }));

            // Scrollable content node
            list_container.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch, // Make buttons stretch to container width
                    overflow: Overflow::clip_y(), // Enable vertical scrolling
                    flex_grow: 1.0, // Allow this node to take remaining space
                    ..default()
                },
                ..default()
            }).with_children(|scrollable_list| {
                for card in upgrade_pool.available_upgrades.iter() {
                    scrollable_list.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: DEBUG_BUTTON_HEIGHT,
                                margin: UiRect::bottom(DEBUG_BUTTON_MARGIN),
                                justify_content: JustifyContent::FlexStart, // Align text to left
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                ..default()
                            },
                            background_color: DEBUG_BUTTON_BG_COLOR.into(),
                            ..default()
                        },
                        DebugUpgradeButton(card.clone()),
                        Name::new(format!("DebugUp: {}", card.name)),
                    )).with_children(|button_content| {
                        button_content.spawn(TextBundle::from_section(
                            // Displaying ID for uniqueness check during debug, can be removed
                            format!("[{}] {} - {}", card.id.0, card.name, card.description), 
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 14.0,
                                color: DEBUG_TEXT_COLOR,
                            },
                        ));
                    });
                }
            });
        });
    });
}

fn debug_menu_button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &DebugUpgradeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut upgrade_chosen_event: EventWriter<UpgradeChosenEvent>,
    mut sound_event_writer: EventWriter<PlaySoundEvent>,
) {
    for (interaction, debug_button_data, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = DEBUG_BUTTON_PRESSED_BG_COLOR.into();
                sound_event_writer.send(PlaySoundEvent(SoundEffect::UpgradeChosen));
                upgrade_chosen_event.send(UpgradeChosenEvent(debug_button_data.0.clone()));
                // Menu does not close automatically; F12 toggles it.
            }
            Interaction::Hovered => {
                *bg_color = DEBUG_BUTTON_HOVER_BG_COLOR.into();
            }
            Interaction::None => {
                *bg_color = DEBUG_BUTTON_BG_COLOR.into();
            }
        }
    }
}

fn despawn_debug_menu_ui(mut commands: Commands, query: Query<Entity, With<DebugMenuUIRoot>>) {
    info!("Despawning Debug Upgrade Menu UI");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}