// --- Imports ---
use bevy::prelude::*;

use crate::{
    map::{calc_mid_room_pos, room::LeaveRoomEvent, X_MAP_LENGTH, Y_MAP_LENGTH},
    menu::AppState,
    PlayerCamera,
};

/// --- Plugin declaration ---
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(room_transit));
    }
}

// --- System-Functions ---
/// Initialize the camera
///
/// Initializes the camera to the middle of the map
fn setup(mut commands: Commands) {
    // camera
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation = Vec3::new(0., 0., 100.0);
    camera.transform.scale = Vec3::new(1.5, 1.5, 1.0);

    let curr_room_pos = calc_mid_room_pos(X_MAP_LENGTH / 2, Y_MAP_LENGTH / 2);
    camera.transform.translation = Vec3::new(curr_room_pos.x, curr_room_pos.y, 100.0);

    let mut camera_entity = commands.spawn_bundle(camera);
    camera_entity.insert(PlayerCamera);
}

/// Move camera on room transition
///
/// Move the camera to the center of the new room if the player transit to a new room
///
/// # Arguments
/// * `ev_leave_room` - Triggered event when the player leaves a room
/// * `camera_query` - Query for the camera entity
fn room_transit(
    mut ev_leave_room: EventReader<LeaveRoomEvent>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    for ev_leave_room in ev_leave_room.iter() {
        let mut transform = camera_query.single_mut();
        let curr_room_pos = calc_mid_room_pos(ev_leave_room.0.x, ev_leave_room.0.y);
        transform.translation = Vec3::new(curr_room_pos.x, curr_room_pos.y, 100.0);
    }
}
