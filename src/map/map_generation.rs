// --- Imports ---
use super::{room::RoomType, room_generation::*, ROOM_DISTANCE, X_MAP_LENGTH, Y_MAP_LENGTH};
use bevy::prelude::*;
use rand::Rng;

// --- Constants ---
const MIN_ROOMS: usize = 8;
const MAX_ROOMS: usize = 12;
const ROOM_CHANCE: f32 = 0.4;

// --- Structs ---
/// Basic room information's to generate the map
#[derive(Clone, Debug)]
pub struct RoomInformation {
    pub room_type: RoomType,
    pub position: RoomPos,
    pub neighbors: Neighbors,
}
impl RoomInformation {
    pub fn new(room_type: RoomType, x: i32, y: i32, neighbors: Neighbors) -> RoomInformation {
        RoomInformation {
            room_type,
            position: RoomPos { x, y },
            neighbors,
        }
    }
}

/// Array based position of a room
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Component)]
pub struct RoomPos {
    pub x: i32,
    pub y: i32,
}
impl RoomPos {
    pub fn new(x: i32, y: i32) -> RoomPos {
        RoomPos { x, y }
    }
    /// Get the position above the current one
    pub fn up(&self) -> RoomPos {
        pos_sum(self, &RoomPos::new(0, 1))
    }
    /// Get the position below the current one
    pub fn down(&self) -> RoomPos {
        pos_sum(self, &RoomPos::new(0, -1))
    }
    /// Get the position to the right of the current one
    pub fn right(&self) -> RoomPos {
        pos_sum(self, &RoomPos::new(1, 0))
    }
    /// Get the position to the left of the current one
    pub fn left(&self) -> RoomPos {
        pos_sum(self, &RoomPos::new(-1, 0))
    }
}

/// Neighbors of a room
#[derive(Clone, Copy, Debug)]
pub struct Neighbors {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}
impl Neighbors {
    pub fn new() -> Neighbors {
        Neighbors {
            top: false,
            bottom: false,
            left: false,
            right: false,
        }
    }
    fn new_with_rooms(room: &RoomPos, rooms: &Vec<RoomPos>) -> Neighbors {
        let mut neighbors = Neighbors::new();

        if rooms.contains(&room.up()) {
            neighbors.top = true;
        }
        if rooms.contains(&room.down()) {
            neighbors.bottom = true;
        }
        if rooms.contains(&room.left()) {
            neighbors.left = true;
        }
        if rooms.contains(&room.right()) {
            neighbors.right = true;
        }

        neighbors
    }
}

/// Create a new map
///
/// Generate a new map with specific configurations and spawn it
pub fn initialize_map(commands: &mut Commands) {
    // Try room generations until a valid map is found
    for _i in 0..1000 {
        // Generate the positions of the rooms
        let rooms = generate_map();
        if rooms.len() >= MIN_ROOMS && rooms.len() <= MAX_ROOMS {
            // Try to add an boss room to the map
            let boss_room = get_boss_room(&rooms);
            if let Some(boss_room) = boss_room {
                // Convert the map from a one dimensional array to a two dimensional array and set some information's
                let map = convert_rooms_to_map(&rooms, &boss_room);
                // Spawn the map
                spawn_map(commands, &map);
                break;
            }
        }
    }
}

/// Generate a one dimensional array of rooms
fn generate_map() -> Vec<RoomPos> {
    // Rooms that will be added to the map
    let mut rooms: Vec<RoomPos> = vec![];
    // Possible rooms which can be added to the map
    let mut queue: Vec<RoomPos> = vec![];

    // Add first room
    let middle_pos: RoomPos = RoomPos::new(X_MAP_LENGTH / 2, Y_MAP_LENGTH / 2);
    rooms.push(middle_pos);
    queue.push(middle_pos.up());
    queue.push(middle_pos.down());
    queue.push(middle_pos.right());
    queue.push(middle_pos.left());

    // Generate rooms
    while !queue.is_empty() {
        // Get next possible room from queue
        let pos = queue.pop().unwrap();

        check_possible_room(&pos, &mut rooms, &mut queue);
    }

    rooms
}

/// Check if a room can be added to the map and try to add it
///
/// The room must have less than 2 neighbors and has an percentage chance to be added
///
/// # Arguments
/// * `room_pos` - Position of the room
/// * `rooms` - Rooms that are already added to the map
/// * `queue` - Possible rooms that could be added to the map but not checked yet
fn check_possible_room(
    room_pos: &RoomPos,
    rooms: &mut Vec<RoomPos>,
    queue: &mut Vec<RoomPos>,
) -> bool {
    // Check if room has more than two neighbors
    if count_neighbors(room_pos, rooms) > 2 {
        return false;
    }

    // Percent chance of adding room to map
    let mut rng = rand::thread_rng();
    if rng.gen::<f32>() < ROOM_CHANCE {
        rooms.push(*room_pos);
        add_possible_neighbors(room_pos, rooms, queue);
    }

    true
}

/// Check surrounding rooms for possible rooms
///
/// If a room is possible, add it to the queue. Only rooms in the map can be added to the queue
///
/// # Arguments
/// * `room_pos` - Position of the room
/// * `rooms` - Rooms that are already added to the map
/// * `queue` - Possible rooms that could be added to the map but not checked yet
fn add_possible_neighbors(room_pos: &RoomPos, rooms: &Vec<RoomPos>, queue: &mut Vec<RoomPos>) {
    for i in 0..4 {
        let pos = match i {
            0 => room_pos.up(),
            1 => room_pos.down(),
            2 => room_pos.right(),
            3 => room_pos.left(),
            _ => panic!("Invalid direction"),
        };

        // Check if room is inside the map borders
        if (pos.x < 0 || pos.x >= X_MAP_LENGTH) || (pos.y < 0 || pos.y >= Y_MAP_LENGTH) {
            continue;
        }

        // If room is already in the map, skip it
        if !rooms.contains(&pos) && !queue.contains(&pos) {
            queue.push(pos);
        }
    }
}

/// Count the number of neighbors of a room which are already in the map
///
/// # Arguments
/// * `room_pos` - Position of the room
/// * `rooms` - Rooms that are already added to the map
///
/// # Returns
/// Number of neighbors of the room which are already in the map
fn count_neighbors(room_pos: &RoomPos, rooms: &Vec<RoomPos>) -> i32 {
    let mut count = 0;
    if rooms.contains(&room_pos.up()) {
        count += 1;
    }
    if rooms.contains(&room_pos.down()) {
        count += 1;
    }
    if rooms.contains(&room_pos.right()) {
        count += 1;
    }
    if rooms.contains(&room_pos.left()) {
        count += 1;
    }

    count
}
/// Add an boss room to the map
///
/// Change one of a room with only one neighbor to an boss room
///
/// # Arguments
/// * `rooms` - The rooms of the map
///
/// # Returns
/// The position of the boss room if a room with only one neighbor was found
fn get_boss_room(rooms: &Vec<RoomPos>) -> Option<RoomPos> {
    // Get all rooms with only one neighbor
    let mut boss_rooms: Vec<RoomPos> = vec![];

    for room in rooms {
        count_neighbors(room, rooms);
        if count_neighbors(room, rooms) == 1
            && *room != RoomPos::new(X_MAP_LENGTH / 2, Y_MAP_LENGTH / 2)
        {
            boss_rooms.push(*room);
        }
    }

    if boss_rooms.is_empty() {
        None
    } else {
        // Get random room from the list
        let mut rng = rand::thread_rng();
        Some(boss_rooms[rng.gen_range(0..boss_rooms.len())])
    }
}

/// Convert room array to two dimensional array map and set some information's for each room
///
/// # Arguments
/// * `rooms` - The rooms of the map
/// * `boss_room` - The position of the boss room
///
/// # Returns
/// The map as a two dimensional array
fn convert_rooms_to_map(rooms: &Vec<RoomPos>, boss_room: &RoomPos) -> Vec<Vec<RoomInformation>> {
    // Initialize two dimensional array map with rooms
    let mut map: Vec<Vec<RoomInformation>> = vec![];

    // Add rooms to map
    for y in 0..Y_MAP_LENGTH {
        let mut row: Vec<RoomInformation> = vec![];
        for x in 0..X_MAP_LENGTH {
            // Calculate the neighbors of the room for further use
            let neighbors = Neighbors::new_with_rooms(&RoomPos::new(x, y), rooms);

            // Add room type, position and neighbors to the room
            if rooms.contains(&RoomPos::new(x, y)) {
                if y == Y_MAP_LENGTH / 2 && x == X_MAP_LENGTH / 2 {
                    row.push(RoomInformation::new(RoomType::Start, x, y, neighbors));
                } else if y == boss_room.y && x == boss_room.x {
                    row.push(RoomInformation::new(RoomType::Boss, x, y, neighbors));
                } else {
                    row.push(RoomInformation::new(RoomType::Normal, x, y, neighbors));
                }
            } else {
                row.push(RoomInformation::new(RoomType::Empty, x, y, neighbors));
            }
        }
        map.push(row);
    }

    map
}

/// Spawn map by spawning every room in the map
///
/// # Arguments
/// * `map` - The map as a two dimensional array
fn spawn_map(commands: &mut Commands, map: &Vec<Vec<RoomInformation>>) {
    for row in map {
        for room in row {
            if room.room_type != RoomType::Empty {
                create_room(
                    commands,
                    room,
                    Vec2::new(
                        room.position.x as f32 * ROOM_DISTANCE,
                        room.position.y as f32 * ROOM_DISTANCE,
                    ),
                );
            }
        }
    }
}

/// Add two positions together
///
/// # Arguments
/// * `pos1` - First position
/// * `pos2` - Second position
///
/// # Returns
/// The sum of the two positions
fn pos_sum(pos1: &RoomPos, pos2: &RoomPos) -> RoomPos {
    RoomPos::new(pos1.x + pos2.x, pos1.y + pos2.y)
}

mod tests {
    #![allow(unused_imports)]
    use super::*;

    /// Test if the position addition works
    #[test]
    fn test_pos_sum() {
        assert_eq!(
            pos_sum(&RoomPos::new(1, 2), &RoomPos::new(3, 4)),
            RoomPos::new(4, 6)
        );
    }

    /// Test if the position of the rooms will be added correctly
    /// Print some statistics about the map
    #[test]
    fn test_count_neighbors() {
        let rooms: Vec<RoomPos> = vec![
            RoomPos::new(5, 4),
            RoomPos::new(6, 4),
            RoomPos::new(8, 2),
            RoomPos::new(5, 3),
        ];

        assert_eq!(count_neighbors(&RoomPos::new(5, 4), &rooms), 2);
        assert_eq!(count_neighbors(&RoomPos::new(6, 4), &rooms), 1);
        assert_eq!(count_neighbors(&RoomPos::new(8, 2), &rooms), 0);
    }

    #[test]
    fn test_room_generation() {
        let trials = 100000;
        let mut count = 0;
        let mut failed = 0;
        let mut boss_room_fails = 0;

        // Try room generations until a valid map is found
        for _i in 0..trials {
            let rooms = generate_map();
            if rooms.len() >= MIN_ROOMS && rooms.len() <= MAX_ROOMS {
                count += rooms.len();

                let boss_room = get_boss_room(&rooms);
                if boss_room.is_none() {
                    boss_room_fails += 1;
                    failed += 1;
                }
            } else {
                failed += 1;
            }
        }

        let average_room_count = count as f32 / (trials - failed) as f32;
        println!("Maximum rooms: {}", MAX_ROOMS);
        println!("Minimum rooms: {}", MIN_ROOMS);
        println!("Possible rooms: {}", X_MAP_LENGTH * Y_MAP_LENGTH);
        println!(
            "Average room count: {0} with {1} map coverage",
            average_room_count,
            (average_room_count / (X_MAP_LENGTH * Y_MAP_LENGTH) as f32 * 100.0),
        );
        println!(
            "Failed percentage: {}",
            failed as f32 / trials as f32 * 100.0
        );
        println!(
            "Boss room fails: {}",
            boss_room_fails as f32 / trials as f32 * 100.0
        );
    }

    /// Test if the map is generated correctly
    #[test]
    fn test_map_generation() {
        for _i in 0..1000 {
            // Generate the positions of the rooms
            let rooms = generate_map();
            if rooms.len() >= MIN_ROOMS && rooms.len() <= MAX_ROOMS {
                let initial_room_count = rooms.len();

                // Try to add an boss room to the map
                let boss_room = get_boss_room(&rooms);
                if let Some(boss_room) = boss_room {
                    // Convert the map from a one dimensional array to a two dimensional array and set some information's
                    let map = convert_rooms_to_map(&rooms, &boss_room);

                    let mut room_count = 0;
                    // Add rooms to map
                    for row in map {
                        for room in row {
                            if room.room_type == RoomType::Normal {
                                print!("{0: <3} |", "X");
                                room_count += 1;
                            } else if room.room_type == RoomType::Start {
                                print!("{0: <3} |", "S");
                                room_count += 1;
                            } else if room.room_type == RoomType::Boss {
                                print!("{0: <3} |", "B");
                                room_count += 1;
                            } else {
                                print!("{0: <3} |", "#");
                            }
                        }
                        println!();
                    }

                    assert!(room_count == initial_room_count);
                    return;
                }
            }
        }

        println!("Map generation test failed");
    }
}
