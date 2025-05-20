use bevy::prelude::*;
use crate::{
    upgrades::{UpgradePool, UpgradeCard},
    game::{AppState, UpgradeChosenEvent, ItemCollectedEvent},
    audio::{PlaySoundEvent, SoundEffect},
    items::{ItemLibrary, ItemId}, // ItemDefinition removed as unused directly here
    skills::{SkillLibrary, SkillId}, // ActiveSkillInstance, SkillDefinition removed as unused directly here
    glyphs::{GlyphLibrary, GlyphId}, // GlyphDefinition removed as unused directly here
    survivor::Survivor, // Changed
};

#[derive(Event)]
pub struct DebugGrantGlyphEvent(pub GlyphId);

#[derive(Event)]
pub struct DebugSocketGlyphEvent {
    pub player_skill_slot_idx: usize,
    pub glyph_slot_idx: usize,
    pub glyph_id_to_socket: GlyphId,
}

pub struct DebugMenuPlugin;

impl Plugin for DebugMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DebugGrantGlyphEvent>()
            .add_event::<DebugSocketGlyphEvent>()
            .add_systems(OnEnter(AppState::DebugUpgradeMenu), setup_debug_menu_ui)
            .add_systems(Update,
                (
                    debug_menu_button_interaction_system,
                    debug_item_button_interaction_system,
                    debug_glyph_button_interaction_system,
                    debug_socket_glyph_button_interaction_system,
                    debug_menu_keyboard_scroll_system,
                )
                .run_if(in_state(AppState::DebugUpgradeMenu))
            )
            .add_systems(Update,
                (
                    handle_debug_grant_glyph.run_if(on_event::<DebugGrantGlyphEvent>()),
                    handle_debug_socket_glyph.run_if(on_event::<DebugSocketGlyphEvent>())
                )
            )
            .add_systems(OnExit(AppState::DebugUpgradeMenu), despawn_debug_menu_ui);
    }
}

#[derive(Component)] struct DebugMenuUIRoot;
#[derive(Component)] struct DebugUpgradeButton(UpgradeCard);
#[derive(Component)] struct DebugItemButton(ItemId);
#[derive(Component)] struct DebugGlyphButton(GlyphId);
#[derive(Component)]
struct DebugSocketGlyphButton {
    player_skill_slot_idx: usize,
    glyph_slot_idx: usize,
    glyph_id_to_socket: GlyphId,
}
#[derive(Component)] struct DebugMenuScrollView;
#[derive(Component)] struct DebugMenuScrollableContent;
#[derive(Component)] struct ScrollOffset(f32);

const DEBUG_BUTTON_HEIGHT: Val = Val::Px(20.0);
const DEBUG_BUTTON_MARGIN: Val = Val::Px(2.0);
const DEBUG_TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const DEBUG_BUTTON_BG_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
const DEBUG_BUTTON_HOVER_BG_COLOR: Color = Color::rgb(0.35, 0.35, 0.35);
const DEBUG_BUTTON_PRESSED_BG_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const DEBUG_SCROLL_AREA_BG_COLOR: Color = Color::rgba(0.12, 0.12, 0.12, 1.0);
const KEYBOARD_SCROLL_SPEED: f32 = 30.0;

fn setup_debug_menu_ui(
    mut commands: Commands, asset_server: Res<AssetServer>,
    upgrade_pool: Res<UpgradePool>, item_library: Res<ItemLibrary>,
    glyph_library: Res<GlyphLibrary>, skill_library: Res<SkillLibrary>,
    player_query: Query<&Survivor>, // Changed
) {
    let player_skills_equipped_glyphs: Vec<(SkillId, Vec<Option<GlyphId>>)> = if let Ok(player) = player_query.get_single() {
        player.equipped_skills.iter().map(|s| (s.definition_id, s.equipped_glyphs.clone())).collect()
    } else { Vec::new() };
    let collected_glyphs_inventory: Vec<GlyphId> = if let Ok(player) = player_query.get_single() { player.collected_glyphs.clone() } else { Vec::new() };

    commands.spawn(( NodeBundle { style: Style { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: Color::rgba(0.0, 0.0, 0.0, 0.90).into(), z_index: ZIndex::Global(50), ..default() }, DebugMenuUIRoot, Name::new("DebugMenuUIRoot"), )).with_children(|parent| {
        parent.spawn(NodeBundle { style: Style { width: Val::Percent(90.0), min_width: Val::Px(900.0), max_width: Val::Px(1400.0), height: Val::Percent(90.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceAround, border: UiRect::all(Val::Px(2.0)), padding: UiRect::all(Val::Px(10.0)), ..default() }, border_color: BorderColor(Color::DARK_GRAY).into(), background_color: Color::rgb(0.05, 0.05, 0.07).into(), ..default()
        }).with_children(|sections_container| {
            sections_container.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, flex_basis: Val::Percent(24.0), margin: UiRect::horizontal(Val::Px(5.0)), ..default() }, ..default() }).with_children(|panel| { panel.spawn(TextBundle::from_section( "UPGRADES", TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 16.0, color: Color::ORANGE_RED,}, ).with_style(Style {margin: UiRect::bottom(Val::Px(8.0)), align_self: AlignSelf::Center, ..default()})); panel.spawn(( NodeBundle { style: Style { overflow: Overflow { y: OverflowAxis::Clip, ..default() }, flex_grow: 1.0, ..default()}, background_color: DEBUG_SCROLL_AREA_BG_COLOR.into(), ..default() }, DebugMenuScrollView, ScrollOffset(0.0), Name::new("UpgradeScroll"), )).with_children(|scroll| { scroll.spawn(( NodeBundle {style: Style {position_type: PositionType::Absolute, width: Val::Percent(100.0), top: Val::Px(0.0), left: Val::Px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch, ..default()}, ..default()}, DebugMenuScrollableContent, Name::new("UpgradeList"), )).with_children(|list| { for card in upgrade_pool.available_upgrades.iter() { list.spawn(( ButtonBundle { style: Style {height: DEBUG_BUTTON_HEIGHT, margin: UiRect::bottom(DEBUG_BUTTON_MARGIN), padding: UiRect::horizontal(Val::Px(5.0)), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, ..default()}, background_color: DEBUG_BUTTON_BG_COLOR.into(), ..default()}, DebugUpgradeButton(card.clone()), Name::new(format!("DbgUp:{}", card.name)), )).with_children(|btn| { btn.spawn(TextBundle::from_section(format!("[{}] {}", card.id.0, card.name), TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 11.0, color: DEBUG_TEXT_COLOR,}));}); } }); }); });
            sections_container.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, flex_basis: Val::Percent(24.0), margin: UiRect::horizontal(Val::Px(5.0)), ..default() }, ..default() }).with_children(|panel| { panel.spawn(TextBundle::from_section( "ITEMS (Grant)", TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 16.0, color: Color::CYAN,}, ).with_style(Style {margin: UiRect::bottom(Val::Px(8.0)), align_self: AlignSelf::Center, ..default()})); panel.spawn(( NodeBundle { style: Style { overflow: Overflow { y: OverflowAxis::Clip, ..default() }, flex_grow: 1.0, ..default()}, background_color: DEBUG_SCROLL_AREA_BG_COLOR.into(), ..default() }, DebugMenuScrollView, ScrollOffset(0.0), Name::new("ItemScroll"), )).with_children(|scroll| { scroll.spawn(( NodeBundle {style: Style {position_type: PositionType::Absolute, width: Val::Percent(100.0), top: Val::Px(0.0), left: Val::Px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch, ..default()}, ..default()}, DebugMenuScrollableContent, Name::new("ItemList"), )).with_children(|list| { for item_def in item_library.items.iter() { list.spawn(( ButtonBundle { style: Style {height: DEBUG_BUTTON_HEIGHT, margin: UiRect::bottom(DEBUG_BUTTON_MARGIN), padding: UiRect::horizontal(Val::Px(5.0)), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, ..default()}, background_color: DEBUG_BUTTON_BG_COLOR.into(), ..default()}, DebugItemButton(item_def.id), Name::new(format!("DbgItem:{}", item_def.name)), )).with_children(|btn| { btn.spawn(TextBundle::from_section(format!("[{}] {}", item_def.id.0, item_def.name), TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 11.0, color: DEBUG_TEXT_COLOR,}));}); } }); }); });
            sections_container.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, flex_basis: Val::Percent(24.0), margin: UiRect::horizontal(Val::Px(5.0)), ..default() }, ..default() }).with_children(|panel| { panel.spawn(TextBundle::from_section( "GLYPHS (Grant to Inv)", TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 16.0, color: Color::LIME_GREEN,}, ).with_style(Style {margin: UiRect::bottom(Val::Px(8.0)), align_self: AlignSelf::Center, ..default()})); panel.spawn(( NodeBundle { style: Style { overflow: Overflow { y: OverflowAxis::Clip, ..default() }, flex_grow: 1.0, ..default()}, background_color: DEBUG_SCROLL_AREA_BG_COLOR.into(), ..default() }, DebugMenuScrollView, ScrollOffset(0.0), Name::new("GlyphGrantScroll"), )).with_children(|scroll| { scroll.spawn(( NodeBundle {style: Style {position_type: PositionType::Absolute, width: Val::Percent(100.0), top: Val::Px(0.0), left: Val::Px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch, ..default()}, ..default()}, DebugMenuScrollableContent, Name::new("GlyphGrantList"), )).with_children(|list| { for glyph_def in glyph_library.glyphs.iter() { list.spawn(( ButtonBundle { style: Style {height: DEBUG_BUTTON_HEIGHT, margin: UiRect::bottom(DEBUG_BUTTON_MARGIN), padding: UiRect::horizontal(Val::Px(5.0)), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, ..default()}, background_color: DEBUG_BUTTON_BG_COLOR.into(), ..default()}, DebugGlyphButton(glyph_def.id), Name::new(format!("DbgGlyphGrant:{}", glyph_def.name)), )).with_children(|btn| { btn.spawn(TextBundle::from_section(format!("[{}] Grant {}", glyph_def.id.0, glyph_def.name), TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 11.0, color: DEBUG_TEXT_COLOR,}));}); } }); }); });
            sections_container.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Column, flex_basis: Val::Percent(24.0), margin: UiRect::horizontal(Val::Px(5.0)), ..default() }, ..default() }).with_children(|panel| { panel.spawn(TextBundle::from_section( "SOCKET GLYPHS", TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 16.0, color: Color::YELLOW,}, ).with_style(Style {margin: UiRect::bottom(Val::Px(8.0)), align_self: AlignSelf::Center, ..default()})); panel.spawn(( NodeBundle { style: Style { overflow: Overflow { y: OverflowAxis::Clip, ..default() }, flex_grow: 1.0, ..default()}, background_color: DEBUG_SCROLL_AREA_BG_COLOR.into(), ..default() }, DebugMenuScrollView, ScrollOffset(0.0), Name::new("GlyphSocketScroll"), )).with_children(|scroll| { scroll.spawn(( NodeBundle {style: Style {position_type: PositionType::Absolute, width: Val::Percent(100.0), top: Val::Px(0.0), left: Val::Px(0.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch, ..default()}, ..default()}, DebugMenuScrollableContent, Name::new("GlyphSocketList"), )).with_children(|list| { for (skill_idx, (skill_id, equipped_glyphs_in_skill)) in player_skills_equipped_glyphs.iter().enumerate() { if let Some(skill_definition) = skill_library.get_skill_definition(*skill_id) { list.spawn(TextBundle::from_section(format!("Skill: {}", skill_definition.name), TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 13.0, color: Color::WHITE,}).with_style(Style {margin: UiRect::top(Val::Px(5.0)), ..default()})); for (glyph_slot_idx, current_glyph_opt) in equipped_glyphs_in_skill.iter().enumerate() { let slot_text = if let Some(current_glyph_id) = current_glyph_opt { glyph_library.get_glyph_definition(*current_glyph_id).map_or("Slot Filled (Unknown)".to_string(), |g| format!("Slot {}: {}", glyph_slot_idx, g.name)) } else { format!("Slot {}: EMPTY", glyph_slot_idx) }; list.spawn(TextBundle::from_section(slot_text, TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 11.0, color: Color::GRAY,}).with_style(Style{ margin: UiRect::left(Val::Px(10.0)), ..default()})); if current_glyph_opt.is_none() { for collected_glyph_id in collected_glyphs_inventory.iter() { if let Some(glyph_to_socket_def) = glyph_library.get_glyph_definition(*collected_glyph_id) { list.spawn(( ButtonBundle { style: Style {height: DEBUG_BUTTON_HEIGHT, margin: UiRect::new(Val::Px(20.0), Val::Px(0.0), Val::Px(0.0),DEBUG_BUTTON_MARGIN), padding: UiRect::horizontal(Val::Px(5.0)), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, ..default()}, background_color: DEBUG_BUTTON_BG_COLOR.into(), ..default()}, DebugSocketGlyphButton { player_skill_slot_idx: skill_idx, glyph_slot_idx, glyph_id_to_socket: *collected_glyph_id }, Name::new(format!("SocketGlyph:{}:S{}:GS{}", glyph_to_socket_def.id.0, skill_idx, glyph_slot_idx)), )).with_children(|btn| { btn.spawn(TextBundle::from_section(format!("Socket '{}'", glyph_to_socket_def.name), TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 10.0, color: DEBUG_TEXT_COLOR,}));}); } } } } } } if collected_glyphs_inventory.is_empty() { list.spawn(TextBundle::from_section("No collected glyphs to socket.", TextStyle {font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 11.0, color: Color::GRAY,}));} }); }); });
        });
    });
}

fn debug_menu_keyboard_scroll_system( keyboard_input: Res<ButtonInput<KeyCode>>, mut scroll_view_query: Query<(&mut ScrollOffset, &Node, &Children, &GlobalTransform), With<DebugMenuScrollView>>, mut content_query: Query<(&Node, &mut Style), With<DebugMenuScrollableContent>>, window_query: Query<&Window, With<bevy::window::PrimaryWindow>>, ) { let Ok(_primary_window) = window_query.get_single() else { return }; let _cursor_pos_option = _primary_window.cursor_position(); for (mut scroll_offset, scroll_view_node, scroll_view_children, _scroll_view_gtransform) in scroll_view_query.iter_mut() { let mut content_entity = None; for &child in scroll_view_children.iter() { if content_query.get(child).is_ok() { content_entity = Some(child); break; } } if let Some(content_e) = content_entity { if let Ok((content_node, mut content_style)) = content_query.get_mut(content_e) { let scroll_view_height = scroll_view_node.size().y; let content_height = content_node.size().y; let mut new_offset = scroll_offset.0; let mut scrolled = false; if keyboard_input.pressed(KeyCode::ArrowUp) { new_offset -= KEYBOARD_SCROLL_SPEED; scrolled = true; } if keyboard_input.pressed(KeyCode::ArrowDown) { new_offset += KEYBOARD_SCROLL_SPEED; scrolled = true; } if scrolled { let max_scroll = (content_height - scroll_view_height).max(0.0); new_offset = new_offset.clamp(0.0, max_scroll); if (scroll_offset.0 - new_offset).abs() > f32::EPSILON { scroll_offset.0 = new_offset; content_style.top = Val::Px(-new_offset); } } } } } }
fn debug_menu_button_interaction_system( mut interaction_query: Query<(&Interaction, &DebugUpgradeButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>, mut upgrade_chosen_event: EventWriter<UpgradeChosenEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (interaction, debug_button_data, mut bg_color) in interaction_query.iter_mut() { match *interaction { Interaction::Pressed => { *bg_color = DEBUG_BUTTON_PRESSED_BG_COLOR.into(); sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted)); upgrade_chosen_event.send(UpgradeChosenEvent(debug_button_data.0.clone())); } Interaction::Hovered => { *bg_color = DEBUG_BUTTON_HOVER_BG_COLOR.into(); } Interaction::None => { *bg_color = DEBUG_BUTTON_BG_COLOR.into(); } } } }
fn debug_item_button_interaction_system( mut interaction_query: Query<(&Interaction, &DebugItemButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>, mut item_collected_event: EventWriter<ItemCollectedEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (interaction, debug_item_button, mut bg_color) in interaction_query.iter_mut() { match *interaction { Interaction::Pressed => { *bg_color = DEBUG_BUTTON_PRESSED_BG_COLOR.into(); sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted)); item_collected_event.send(ItemCollectedEvent(debug_item_button.0)); } Interaction::Hovered => { *bg_color = DEBUG_BUTTON_HOVER_BG_COLOR.into(); } Interaction::None => { *bg_color = DEBUG_BUTTON_BG_COLOR.into(); } } } }
fn debug_glyph_button_interaction_system( mut interaction_query: Query<(&Interaction, &DebugGlyphButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>, mut grant_glyph_event_writer: EventWriter<DebugGrantGlyphEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (interaction, debug_glyph_button, mut bg_color) in interaction_query.iter_mut() { match *interaction { Interaction::Pressed => { *bg_color = DEBUG_BUTTON_PRESSED_BG_COLOR.into(); sound_event_writer.send(PlaySoundEvent(SoundEffect::SoulCollect)); grant_glyph_event_writer.send(DebugGrantGlyphEvent(debug_glyph_button.0)); } Interaction::Hovered => { *bg_color = DEBUG_BUTTON_HOVER_BG_COLOR.into(); } Interaction::None => { *bg_color = DEBUG_BUTTON_BG_COLOR.into(); } } } }
fn debug_socket_glyph_button_interaction_system( mut interaction_query: Query<(&Interaction, &DebugSocketGlyphButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>, mut socket_glyph_event_writer: EventWriter<DebugSocketGlyphEvent>, mut sound_event_writer: EventWriter<PlaySoundEvent>,) { for (interaction, button_data, mut bg_color) in interaction_query.iter_mut() { match *interaction { Interaction::Pressed => { *bg_color = DEBUG_BUTTON_PRESSED_BG_COLOR.into(); sound_event_writer.send(PlaySoundEvent(SoundEffect::OmenAccepted)); socket_glyph_event_writer.send(DebugSocketGlyphEvent { player_skill_slot_idx: button_data.player_skill_slot_idx, glyph_slot_idx: button_data.glyph_slot_idx, glyph_id_to_socket: button_data.glyph_id_to_socket, }); } Interaction::Hovered => { *bg_color = DEBUG_BUTTON_HOVER_BG_COLOR.into(); } Interaction::None => { *bg_color = DEBUG_BUTTON_BG_COLOR.into(); } } } }
fn handle_debug_grant_glyph( mut events: EventReader<DebugGrantGlyphEvent>, mut player_query: Query<&mut Survivor>,) { if let Ok(mut player) = player_query.get_single_mut() { for event in events.read() { if !player.collected_glyphs.contains(&event.0) { player.collected_glyphs.push(event.0); } } } } // Changed
fn handle_debug_socket_glyph( mut events: EventReader<DebugSocketGlyphEvent>, mut player_query: Query<&mut Survivor>,) { if let Ok(mut player) = player_query.get_single_mut() { for event in events.read() { if let Some(collected_glyph_index) = player.collected_glyphs.iter().position(|&id| id == event.glyph_id_to_socket) { if let Some(skill_instance) = player.equipped_skills.get_mut(event.player_skill_slot_idx) { if event.glyph_slot_idx < skill_instance.equipped_glyphs.len() && skill_instance.equipped_glyphs[event.glyph_slot_idx].is_none() { skill_instance.equipped_glyphs[event.glyph_slot_idx] = Some(event.glyph_id_to_socket); player.collected_glyphs.remove(collected_glyph_index); } } } } } } // Changed
fn despawn_debug_menu_ui(mut commands: Commands, query: Query<Entity, With<DebugMenuUIRoot>>) { for entity in query.iter() { commands.entity(entity).despawn_recursive(); } }