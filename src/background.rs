use bevy::prelude::*;
use crate::camera_systems::MainCamera;
use crate::game::AppState;

pub const BACKGROUND_TILE_SIZE: f32 = 2048.0;
const BACKGROUND_Z: f32 = -10.0;
const GRID_DIMENSION: i32 = 5; 
const NUM_TILES: usize = (GRID_DIMENSION * GRID_DIMENSION) as usize;
// Shift the grid when camera moves this fraction of a tile size past the center tile's edge
const GRID_SHIFT_THRESHOLD_FACTOR: f32 = 0.45; // Previously effectively 0.5

#[derive(Component)]
struct BackgroundTile;

#[derive(Resource)]
struct BackgroundGrid {
    tiles: [Entity; NUM_TILES],
    grid_logical_center: Vec2,
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), setup_background)
            .add_systems(Update, infinite_scroll_background.run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), cleanup_background);
    }
}

fn setup_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut tiles = [Entity::PLACEHOLDER; NUM_TILES];
    let grid_half_span_offset = (GRID_DIMENSION as f32 - 1.0) / 2.0; 

    for i in 0..GRID_DIMENSION { 
        for j in 0..GRID_DIMENSION { 
            let x_pos = (j as f32 - grid_half_span_offset) * BACKGROUND_TILE_SIZE;
            let y_pos = (i as f32 - grid_half_span_offset) * BACKGROUND_TILE_SIZE;
            
            let tile_entity = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("sprites/cyclopean_ruins_tile_placeholder.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(BACKGROUND_TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x_pos, y_pos, BACKGROUND_Z),
                    ..default()
                },
                BackgroundTile,
                Name::new(format!("BackgroundTile_{}_{}", i, j)),
            )).id();
            tiles[(i * GRID_DIMENSION + j) as usize] = tile_entity;
        }
    }
    commands.insert_resource(BackgroundGrid { tiles, grid_logical_center: Vec2::ZERO });
}

fn infinite_scroll_background(
    camera_query: Query<&Transform, With<MainCamera>>,
    mut background_grid: ResMut<BackgroundGrid>,
    mut tile_transforms: Query<&mut Transform, (With<BackgroundTile>, Without<MainCamera>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return; };
    let camera_pos = camera_transform.translation.truncate();

    let dx = camera_pos.x - background_grid.grid_logical_center.x;
    let dy = camera_pos.y - background_grid.grid_logical_center.y;

    let threshold = BACKGROUND_TILE_SIZE * GRID_SHIFT_THRESHOLD_FACTOR;
    
    let mut shift_x_tiles_count = 0i32;
    let mut shift_y_tiles_count = 0i32;

    if dx > threshold {
        shift_x_tiles_count = ((dx - threshold) / BACKGROUND_TILE_SIZE).ceil() as i32;
         if shift_x_tiles_count == 0 && dx > threshold { shift_x_tiles_count = 1; } // Ensure at least one tile shift if over threshold
    } else if dx < -threshold {
        shift_x_tiles_count = ((dx + threshold) / BACKGROUND_TILE_SIZE).floor() as i32;
        if shift_x_tiles_count == 0 && dx < -threshold { shift_x_tiles_count = -1; }
    }

    if dy > threshold {
        shift_y_tiles_count = ((dy - threshold) / BACKGROUND_TILE_SIZE).ceil() as i32;
        if shift_y_tiles_count == 0 && dy > threshold { shift_y_tiles_count = 1; }
    } else if dy < -threshold {
        shift_y_tiles_count = ((dy + threshold) / BACKGROUND_TILE_SIZE).floor() as i32;
        if shift_y_tiles_count == 0 && dy < -threshold { shift_y_tiles_count = -1; }
    }
    
    if shift_x_tiles_count != 0 || shift_y_tiles_count != 0 {
        let grid_total_span = GRID_DIMENSION as f32 * BACKGROUND_TILE_SIZE;
        let half_grid_span_from_center = (GRID_DIMENSION as f32 / 2.0) * BACKGROUND_TILE_SIZE;

        // Update the logical center of the grid first
        let new_grid_center_x = background_grid.grid_logical_center.x + shift_x_tiles_count as f32 * BACKGROUND_TILE_SIZE;
        let new_grid_center_y = background_grid.grid_logical_center.y + shift_y_tiles_count as f32 * BACKGROUND_TILE_SIZE;

        for tile_entity_id in background_grid.tiles.iter() {
            if let Ok(mut tile_transform) = tile_transforms.get_mut(*tile_entity_id) {
                // Adjust tile position based on how much the logical center shifted
                tile_transform.translation.x += shift_x_tiles_count as f32 * BACKGROUND_TILE_SIZE;
                tile_transform.translation.y += shift_y_tiles_count as f32 * BACKGROUND_TILE_SIZE;

                // Wrap tiles that are now too far from the NEW logical center
                if tile_transform.translation.x < new_grid_center_x - half_grid_span_from_center {
                    tile_transform.translation.x += grid_total_span;
                } else if tile_transform.translation.x >= new_grid_center_x + half_grid_span_from_center { // Use >= for upper bound
                    tile_transform.translation.x -= grid_total_span;
                }
                
                if tile_transform.translation.y < new_grid_center_y - half_grid_span_from_center {
                    tile_transform.translation.y += grid_total_span;
                } else if tile_transform.translation.y >= new_grid_center_y + half_grid_span_from_center {
                    tile_transform.translation.y -= grid_total_span;
                }
            }
        }
        // Officially update the grid's logical center
        background_grid.grid_logical_center.x = new_grid_center_x;
        background_grid.grid_logical_center.y = new_grid_center_y;
    }
}


fn cleanup_background(mut commands: Commands, query: Query<Entity, With<BackgroundTile>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
//Placeholder for fleshy_landscape_tile_placeholder.png if used
//The current code only uses one background tile, so background_tile2.png is not used.