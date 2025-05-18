use bevy::prelude::*;
use crate::game::AppState;

const DAMAGE_TEXT_LIFETIME_SECONDS: f32 = 0.75;
const DAMAGE_TEXT_SPEED: f32 = 60.0;
// Removed unused DAMAGE_TEXT_FADE_SPEED

pub struct VisualEffectsPlugin;

impl Plugin for VisualEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            animate_damage_text_system.run_if(in_state(AppState::InGame))
        );
    }
}

#[derive(Component)]
pub struct DamageTextEffect {
    pub spawn_time: f32,
    pub velocity: Vec2,
}

pub fn spawn_damage_text(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3, 
    damage_amount: i32,
    time: &Res<Time>, 
) {
    let random_offset_x = (rand::random::<f32>() - 0.5) * 20.0; 

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                damage_amount.to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0, 
                    color: Color::rgb(1.0, 0.8, 0.8), 
                },
            ),
            transform: Transform::from_translation(position + Vec3::new(random_offset_x, 10.0, 5.0)),
            ..default()
        },
        DamageTextEffect {
            spawn_time: time.elapsed_seconds(),
            velocity: Vec2::new(random_offset_x * 0.5, DAMAGE_TEXT_SPEED), 
        },
        Name::new("DamageText"),
    ));
}

fn animate_damage_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &DamageTextEffect, &mut Transform, &mut Text)>,
) {
    let current_time = time.elapsed_seconds();
    for (entity, effect_data, mut transform, mut text_component) in query.iter_mut() {
        let time_alive = current_time - effect_data.spawn_time;

        if time_alive > DAMAGE_TEXT_LIFETIME_SECONDS {
            commands.entity(entity).despawn_recursive(); // Use despawn_recursive for safety
            continue;
        }

        transform.translation.y += effect_data.velocity.y * time.delta_seconds();
        transform.translation.x += effect_data.velocity.x * time.delta_seconds();
        
        if let Some(section) = text_component.sections.get_mut(0) {
            let alpha_progress = (time_alive / DAMAGE_TEXT_LIFETIME_SECONDS).powf(2.0); 
            section.style.color.set_a((1.0 - alpha_progress).max(0.0));
        }
    }
}