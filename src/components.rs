use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Damage(pub i32);

#[derive(Component)]
pub struct Cooldown { // Currently unused
    pub timer: Timer,
}

#[derive(Component)]
pub struct Target(pub Option<Entity>); // Currently unused

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}