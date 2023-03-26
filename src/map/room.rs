// --- Imports ---
use bevy::prelude::*;

use crate::{
    menu::AppState,
    spawnable::{
        enemy::{
            behavior::{Enemy, EnemySlainEvent},
            enemy_types::{spawn_enemy_type, EnemyResource},
        },
        movement::Collider,
        weapon::WeaponResource,
    },
};

use super::{
    calc_mid_room_pos,
    map_generation::{Neighbors, RoomInformation, RoomPos},
    room_generation::Spawner,
    ROOM_HEIGHT, TILE_SIZE, X_ROOM_LENGTH, Y_ROOM_LENGTH,
};

// --- Plugin declaration ---
pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RoomFinishedEvent>()
            .add_event::<LeaveRoomEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(finished_room)
                    .with_system(open_doors)
                    .with_system(enemy_slain.after("despawn"))
                    .with_system(room_transit),
            );
    }
}

// --- Structs ---

/// Type of the room
///
/// Rooms are distinguished by following room types
///
/// * `Start` - First room where the player will spawn
/// * `Empty` - Empty room so the player can rest for a wile
/// * `Normal` - Standard room with enemies
/// * `Boss` - Room where the boss will wait to complete the stage
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Component)]
pub enum RoomType {
    Start,
    Empty,
    Normal,
    Boss,
}

/// Status of the room
///
/// Every room has a status for further calculations
///
/// * `Closed` - Room is closed and can be opened by the player
/// * `Open`   - Room is open and can be completed by the player
/// * `Finished` - Room is finished and can't be opened again
#[derive(Clone, Component, PartialEq, PartialOrd, Debug)]
pub enum RoomStatus {
    Closed,
    Active,
    Finished,
}

/// Type of an specific tile of an room
///
/// Every tile of an room has an different type to distinguish it
///
/// * `Wall` - Wall tile which blocks the player
/// * `Empty` - No tile
/// * `Door` - Door tile which will be opened if a room is completed
/// * `Spawner` - Spawner tile to spawn a specific enemy
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Component)]
pub enum TileType {
    Empty,
    Wall,
    Door,
    Spawner,
}

/// Transition direction of the room transition
///
/// To move from one room to another the player has to move in a specific direction
///
/// * `Up` - Move up
/// * `Down` - Move down
/// * `Left` - Move left
/// * `Right` - Move right
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Component)]
pub enum TransitionDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Room transition marker component
///
/// This marker is used to mark the room transition
#[derive(Component)]
pub struct RoomTransition;

/// Door marker component
///
/// This marker is used to mark the doors of an room
#[derive(Component)]
pub struct Door;

/// Door marker component
///
/// This marker is used to mark the doors of an room
#[derive(Component)]
pub struct Room;

// --- Events ---
/// Event to signal that the room is finished
///
/// This event is sent when the player killed all the enemies in there
pub struct RoomFinishedEvent(pub RoomPos);

/// Event to signal that the player left the room
///
/// This event is sent when the player left the room to make the new room active
pub struct LeaveRoomEvent(pub RoomPos, pub TransitionDirection);

// --- Bundles ---

/// Bundle with basic components for a room
#[derive(Bundle)]
struct RoomBundle {
    room_type: RoomType,
    room_status: RoomStatus,
    position: RoomPos,
    room: Room,
}

// --- Functions
/// Spawns the room entity
///
/// Spawns an room entity with some general information's about it
///
/// # Arguments
/// `commands` - Commands to spawn the room
/// `room_information` - Information about the room
pub fn spawn_room(commands: &mut Commands, room_information: &RoomInformation) {
    // If the room is a start room set it as active
    let room_status = match room_information.room_type {
        RoomType::Start => RoomStatus::Active,
        _ => RoomStatus::Closed,
    };

    commands.spawn_bundle(RoomBundle {
        room_type: room_information.room_type,
        room_status,
        position: RoomPos::new(room_information.position.x, room_information.position.y),
        room: Room,
    });

    // Spawn transitions
    spawn_room_transition(
        commands,
        &room_information.neighbors,
        &room_information.position,
    )
}

/// Spawn a room transition at a edge of a room, if it has a neighbor there.
///
/// # Arguments
/// `commands` - Commands to spawn the room
/// `neighbors` - Neighbors of the room
/// `room_pos` - Position of the room
fn spawn_room_transition(commands: &mut Commands, neighbors: &Neighbors, room_pos: &RoomPos) {
    let mid_pos = calc_mid_room_pos(room_pos.x, room_pos.y);

    // Calculate the distance of the center of the room to the edge where the transition should be spawned
    let x_distance = (X_ROOM_LENGTH + 1) as f32 * TILE_SIZE / 2.;
    let y_distance = (Y_ROOM_LENGTH + 1) as f32 * TILE_SIZE / 2.;

    for i in 0..4 {
        let mut position = Vec3::new(mid_pos.x, mid_pos.y, ROOM_HEIGHT - 1.);
        let mut scale = Vec3::new(TILE_SIZE, TILE_SIZE, 0.0);

        let transition_direction: TransitionDirection;
        let destination: RoomPos;

        // Set configuration for the transition
        match (i, neighbors) {
            (0, &Neighbors { top: true, .. }) => {
                position.y += y_distance;
                scale.x *= X_ROOM_LENGTH as f32;
                transition_direction = TransitionDirection::Up;
                destination = room_pos.up();
            }
            (1, &Neighbors { bottom: true, .. }) => {
                position.y -= y_distance;
                scale.x *= X_ROOM_LENGTH as f32;
                transition_direction = TransitionDirection::Down;
                destination = room_pos.down();
            }
            (2, &Neighbors { left: true, .. }) => {
                position.x -= x_distance;
                scale.y *= Y_ROOM_LENGTH as f32;
                transition_direction = TransitionDirection::Left;
                destination = room_pos.left();
            }
            (3, &Neighbors { right: true, .. }) => {
                position.x += x_distance;
                scale.y *= Y_ROOM_LENGTH as f32;
                transition_direction = TransitionDirection::Right;
                destination = room_pos.right();
            }
            _ => {
                continue;
            }
        }

        spawn_room_transition_tile(commands, position, scale, destination, transition_direction);
    }
}

/// Spawns a room transition tile
///
/// # Arguments
/// `commands` - Commands to spawn the room
/// `position` - Position of the transition
/// `scale` - Scale of the transition
/// `destination` - Destination room position of the transition
/// `transition_direction` - Direction of the transition
fn spawn_room_transition_tile(
    commands: &mut Commands,
    position: Vec3,
    scale: Vec3,
    destination: RoomPos,
    transition_direction: TransitionDirection,
) {
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: position,
                scale,
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(destination)
        .insert(transition_direction)
        .insert(Name::new("RoomTransition"))
        .insert(Collider::RoomTransition);
}

// --- Events ---
/// Room transition event of a room
///
/// The room will activate it self if its not already active
/// and finish it self directly if no enemy will be spawned`
fn room_transit(
    mut commands: Commands,
    mut ev_leave_room: EventReader<LeaveRoomEvent>,
    mut ev_room_finished: EventWriter<RoomFinishedEvent>,
    room_query: Query<(&RoomStatus, &RoomPos, Entity), With<Room>>,
    enemy_query: Query<(&RoomPos, Entity), With<Enemy>>,
    spawner_query: Query<(&Transform, &Spawner, &RoomPos, Entity)>,
    enemy_res: Res<EnemyResource>,
    weapon_res: Res<WeaponResource>,
) {
    for ev_leave_room in ev_leave_room.iter() {
        // Go through all rooms and check if its the specified room of the event and is not already active
        for (room_status, room_pos, room) in room_query.iter() {
            if *room_pos == ev_leave_room.0 && *room_status == RoomStatus::Closed {
                let mut r = commands.entity(room);

                // Activate the room by changing the room status
                r.remove::<RoomStatus>();
                r.insert(RoomStatus::Active);

                // Check for living enemies or enemies that will be spawned.
                // If no enemy is living or will be spawned finish the room directly
                if !check_for_enemies(&enemy_query, room_pos, None)
                    && !spawn_enemies(
                        &mut commands,
                        &spawner_query,
                        room_pos,
                        &enemy_res,
                        &weapon_res,
                    )
                {
                    ev_room_finished.send(RoomFinishedEvent(*room_pos))
                };
            }
        }
    }
}

/// Spawn enemies at the spawner position and return true if enemies were spawned
fn spawn_enemies(
    commands: &mut Commands,
    spawner_query: &Query<(&Transform, &Spawner, &RoomPos, Entity)>,
    room_pos: &RoomPos,
    enemy_res: &Res<EnemyResource>,
    weapon_res: &Res<WeaponResource>,
) -> bool {
    let mut enemies_spawned = false;

    // Go through all spawners and spawn enemies
    for (transform, spawner, spawner_room_pos, entity) in spawner_query.iter() {
        if *room_pos == *spawner_room_pos {
            spawn_enemy_type(
                commands,
                enemy_res,
                weapon_res,
                &spawner.enemy_type,
                Vec2::new(transform.translation.x, transform.translation.y),
                *room_pos,
            );

            commands.entity(entity).remove::<Spawner>();

            if !enemies_spawned {
                enemies_spawned = true;
            }
        }
    }

    enemies_spawned
}

/// Finish room if no enemy is living
///
/// If an enemy was slain check if all enemies are dead and if so finish the room
fn enemy_slain(
    mut ev_enemy_slain: EventReader<EnemySlainEvent>,
    mut ev_room_finished: EventWriter<RoomFinishedEvent>,
    room_query: Query<(&RoomStatus, &RoomPos), With<Room>>,
    enemy_query: Query<(&RoomPos, Entity), With<Enemy>>,
) {
    // Go through all rooms and check if its the specified room of the slain event
    for ev_enemy_slain in ev_enemy_slain.iter() {
        for (room_status, room_pos) in room_query.iter() {
            // Check if the room is active and if no enemy is living that's connected to the room, finish the room
            if *room_status == RoomStatus::Active
                && *room_pos == ev_enemy_slain.0
                && !check_for_enemies(&enemy_query, room_pos, Some(ev_enemy_slain.1))
            {
                ev_room_finished.send(RoomFinishedEvent(ev_enemy_slain.0));
            }
        }
    }
}

/// Check if theres no enemies living that are spawned inside the given room
fn check_for_enemies(
    enemy_query: &Query<(&RoomPos, Entity), With<Enemy>>,
    room_pos: &RoomPos,
    killed_enemy: Option<Entity>,
) -> bool {
    for (enemy_room_pos, entity) in enemy_query.iter() {
        if killed_enemy.is_some() && (killed_enemy.unwrap() == entity) {
            continue;
        }
        if enemy_room_pos == room_pos {
            return true;
        }
    }
    false
}

/// Finish room if event is triggered
///
/// The room will finish if the event is triggered by changing the room status
fn finished_room(
    mut commands: Commands,
    mut ev_room_finished: EventReader<RoomFinishedEvent>,
    query: Query<(&RoomStatus, &RoomPos, Entity), With<Room>>,
) {
    for ev_room_finished in ev_room_finished.iter() {
        for (room_status, room_pos, room) in query.iter() {
            if room_pos == &ev_room_finished.0 && *room_status == RoomStatus::Active {
                let mut r = commands.entity(room);
                r.remove::<RoomStatus>();
                r.insert(RoomStatus::Finished);
            }
        }
    }
}

/// Opens the doors of a room if an room was finished
fn open_doors(
    mut commands: Commands,
    mut room_finished_event: EventReader<RoomFinishedEvent>,
    query: Query<(&RoomPos, Entity), With<Door>>,
) {
    for ev in room_finished_event.iter() {
        for (room_pos, door) in query.iter() {
            if room_pos == &ev.0 {
                let mut entity = commands.entity(door);
                entity
                    .remove::<Door>()
                    .remove::<Collider>()
                    .remove::<Sprite>();
                entity.insert(Sprite {
                    color: Color::rgb(0.0, 0.0, 0.0),
                    ..Default::default()
                });
            }
        }
    }
}
