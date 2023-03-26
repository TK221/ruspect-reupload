// --- Imports ---
use crate::menu::AppState;
use bevy::prelude::*;

// --- Plugins imports ---
pub mod map_generation;
pub mod room;
pub mod room_generation;

use self::map_generation::{initialize_map, RoomPos};
use self::room::{RoomFinishedEvent, RoomPlugin};

// --- Plugin declaration ---
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RoomPlugin).add_system_set(
            SystemSet::on_enter(AppState::InGame).with_system(start_map_generation),
        );
    }
}

// --- Constants ---
/// Distance between rooms
pub const ROOM_DISTANCE: f32 = 10.0 * TILE_SIZE * (X_ROOM_LENGTH + Y_ROOM_LENGTH) as f32;
/// Horizontal length of the map
pub const X_MAP_LENGTH: i32 = 9;
/// Vertical length of the map
pub const Y_MAP_LENGTH: i32 = 9;

/// Horizontal length of the room
pub const X_ROOM_LENGTH: usize = 25;
/// Vertical length of the room
pub const Y_ROOM_LENGTH: usize = 15;
/// Layer where the room will be shown
pub const ROOM_HEIGHT: f32 = -1.0;
/// The size of a tile
pub const TILE_SIZE: f32 = 60.0;

// --- Systems ---
/// Start the map generation
///
/// This system is called when the player enters the game state and will initialize the map
fn start_map_generation(
    mut commands: Commands,
    mut ev_room_finished: EventWriter<RoomFinishedEvent>,
) {
    initialize_map(&mut commands);

    ev_room_finished.send(RoomFinishedEvent(RoomPos::new(
        X_MAP_LENGTH / 2,
        Y_MAP_LENGTH / 2,
    )));
}

// --- Functions ---
/// Calculate the center position of a specified room position
///
/// # Arguments
/// * `x` - The x position of the room
/// * `y` - The y position of the room
///
/// # Returns
/// The center position of the room
pub fn calc_mid_room_pos(x: i32, y: i32) -> Vec2 {
    Vec2::new(
        x as f32 * ROOM_DISTANCE + X_ROOM_LENGTH as f32 * TILE_SIZE / 2.0 - TILE_SIZE / 2.0,
        y as f32 * ROOM_DISTANCE + Y_ROOM_LENGTH as f32 * TILE_SIZE / 2.0 - TILE_SIZE / 2.0,
    )
}
