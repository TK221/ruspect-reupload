use crate::{
    menu::AppState,
    spawnable::{behavior::Spawnable, movement::Movement},
};
use bevy::prelude::*;

pub struct BulletPlugin;

// --- Execute systems ---
impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(bullet_movement_and_collision)
                .before("step2"),
        );
    }
}

// --- Components and Structs ---
/// Bullet component
#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub direction: Vec2,
    pub damage: f32,
    pub is_player: bool,
    pub knockback_strength: f32,
    pub knockback_duration: f32,
    pub hitstun_duration: f32,
}

// --- System-Functions ---
// Adds movement to bullets
fn bullet_movement_and_collision(
    entity_query: Query<(&Bullet, Entity, &Transform, &Spawnable), With<Bullet>>,
    mut commands: Commands,
) {
    for (bullet, bullet_entity, transform, spawnable) in entity_query.iter() {
        if !(spawnable.despawn || bullet.direction.x == 0.0 && bullet.direction.y == 0.0) {
            commands.entity(bullet_entity).insert(Movement {
                direction: Vec2::new(bullet.direction.x, bullet.direction.y).normalize(),
                transform: *transform,
                speed: bullet.speed,
            });
        }
    }
}
