// --- Imports ---
use super::{
    map_generation::{Neighbors, RoomInformation, RoomPos},
    room::{spawn_room, Door, RoomType, TileType},
    ROOM_HEIGHT, TILE_SIZE, X_ROOM_LENGTH, Y_ROOM_LENGTH,
};
use bevy::prelude::*;
use rand::Rng;
use std::{
    fs::{read_dir, File},
    io::{BufRead, BufReader},
};

use crate::spawnable::{enemy::enemy_types::EnemyType, movement::Collider};

// --- Components ---
/// Component to identify a room
#[derive(Component, Debug, Clone)]
pub struct Spawner {
    pub enemy_type: EnemyType,
}

// --- Room generation ---
/// Create a new random room with the given information's
///
/// A random room will be created with the specified position and room type
///
/// # Arguments
/// `commands` - The commands to add the room to the world
/// `room_information` - The information of the room
/// `offset` - The offset of the room
pub fn create_room(commands: &mut Commands, room_information: &RoomInformation, offset: Vec2) {
    let mut map = create_room_map(room_information.room_type);
    map = add_walls(map, room_information.neighbors);

    spawn_room_map(
        commands,
        map,
        offset,
        &room_information.position,
        &room_information.room_type,
    );
    spawn_room(commands, room_information)
}

/// Generate a random room map
///
/// Get and random room of the defined rooms and convert it to an two dimensional array with specific tile types
///
/// # Arguments
/// `room_type` - The type of the room
///
/// # Returns
/// A two dimensional array with the room's tiles
fn create_room_map(room_type: RoomType) -> Vec<Vec<TileType>> {
    let file = get_random_room(room_type);

    let mut map = vec![vec![TileType::Empty; X_ROOM_LENGTH - 2]; Y_ROOM_LENGTH - 2];

    for (y, line) in BufReader::new(file).lines().enumerate() {
        if let Ok(line) = line {
            for (x, character) in line.chars().enumerate() {
                match character {
                    'X' => map[Y_ROOM_LENGTH - y - 3][x] = TileType::Wall,
                    'S' => map[Y_ROOM_LENGTH - y - 3][x] = TileType::Spawner,
                    _ => map[Y_ROOM_LENGTH - y - 3][x] = TileType::Empty,
                }
                if x >= X_ROOM_LENGTH - 3 {
                    break;
                };
            }
            if y >= Y_ROOM_LENGTH - 3 {
                break;
            };
        }
    }

    map
}

/// Get a random room by its room type
///
/// Read the room files in the `rooms` folder and return a random room file
///
/// # Arguments
/// * `room_type` - The room type to get a random room for
///
/// # Returns
/// The random room file
fn get_random_room(room_type: RoomType) -> File {
    // Build directory path
    let mut directory = "assets/rooms/".to_string();
    match room_type {
        RoomType::Start => directory.push_str("start"),
        RoomType::Empty => directory.push_str("empty"),
        RoomType::Normal => directory.push_str("normal"),
        RoomType::Boss => directory.push_str("boss"),
    }

    // Read all files in the directory
    let file_list_iter = read_dir(directory.clone());
    if let Err(err) = file_list_iter {
        panic!("Could not read directory: {}", err);
    }

    // Add Files to the list
    let mut file_list = Vec::new();
    for file in file_list_iter.unwrap() {
        if let Err(err) = file {
            panic!("Could not read file: {}", err);
        }

        file_list.push(file.unwrap().path());
    }

    // Panic if there are no files
    if file_list.is_empty() {
        panic!("No files found in directory: {}", directory);
    }

    // Get a random file
    let mut rng = rand::thread_rng();
    let file_path = file_list[rng.gen_range(0..file_list.len())].to_str();
    if file_path.is_none() {
        panic!("Could not convert file path to string");
    }

    // Open the file and return it
    let file = File::open(file_path.unwrap());
    if let Err(err) = file {
        panic!("Could not open file: {}", err);
    }
    file.unwrap()
}

/// Surround the room with walls and doors on each side
///
/// # Arguments
/// * `map` - The room map to add walls to
/// * `neighbors` - The neighbors of the room
///
/// # Returns
/// The room map with walls and doors
fn add_walls(map: Vec<Vec<TileType>>, neighbors: Neighbors) -> Vec<Vec<TileType>> {
    // Initialize a new two dimensional array with the same size as the map plus one row on each side
    let mut new_map = vec![vec![TileType::Wall; X_ROOM_LENGTH]; Y_ROOM_LENGTH];

    // Copy the map into the new map
    for y in 0..Y_ROOM_LENGTH - 2 {
        for x in 0..X_ROOM_LENGTH - 2 {
            new_map[y + 1][x + 1] = map[y][x];
        }
    }

    // Add on door on each side
    if neighbors.top {
        new_map[Y_ROOM_LENGTH - 1][X_ROOM_LENGTH / 2] = TileType::Door;
    }
    if neighbors.bottom {
        new_map[0][X_ROOM_LENGTH / 2] = TileType::Door;
    }
    if neighbors.left {
        new_map[Y_ROOM_LENGTH / 2][0] = TileType::Door;
    }
    if neighbors.right {
        new_map[Y_ROOM_LENGTH / 2][X_ROOM_LENGTH - 1] = TileType::Door;
    }

    new_map
}

// --- Room spawning ---
/// Spawn a room with the given information
///
/// A room will be spawned with the specified position and room type
///
/// # Arguments
/// * `commands` - The commands to spawn the room with
/// * `map` - The room map to spawn
/// * `offset` - The offset to spawn the room at
/// * `position` - The position of the room
/// * `room_type` - The type of the room
fn spawn_room_map(
    commands: &mut Commands,
    map: Vec<Vec<TileType>>,
    offset: Vec2,
    room_pos: &RoomPos,
    room_type: &RoomType,
) {
    // iterate over map and spawn the tiles
    for (y, row) in map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let x_pos = x as f32 * TILE_SIZE + offset.x;
            let y_pos = y as f32 * TILE_SIZE + offset.y;

            spawn_tile(
                commands,
                Vec3::new(x_pos, y_pos, ROOM_HEIGHT),
                *tile,
                room_pos,
                room_type,
            );
        }
    }
}

/// Spawn a tile with the given information
///
/// A tile will be spawned with the specified position, tile type and room type
///
/// # Arguments
/// * `commands` - The commands to spawn the tile with
/// * `position` - The position of the tile
/// * `tile_type` - The type of the tile
/// * `room_pos` - The position of the room
/// * `room_type` - The type of the room
fn spawn_tile(
    commands: &mut Commands,
    position: Vec3,
    tile_type: TileType,
    room_pos: &RoomPos,
    room_type: &RoomType,
) {
    let color: Color;
    let name: String;
    let collider: bool;

    // Set color, name and collider based on tile type
    match tile_type {
        TileType::Empty => {
            color = Color::rgb(0.0, 0.0, 0.0);
            name = "Floor".to_string();
            collider = false;
        }
        TileType::Wall => {
            color = Color::rgb(0.8, 0.8, 0.8);
            name = "Wall".to_string();
            collider = true;
        }
        TileType::Door => {
            color = Color::rgb(0.33, 0.18, 0.07);

            name = "Door".to_string();
            collider = true;
        }
        TileType::Spawner => {
            color = Color::rgb(0.0, 0.0, 0.0);
            name = "Spawner".to_string();
            collider = false;
        }
    }

    // Spawn the tile
    let mut tile = commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: position,
            scale: Vec3::new(TILE_SIZE, TILE_SIZE, 0.),
            ..Default::default()
        },
        sprite: Sprite {
            color,
            ..Default::default()
        },
        ..Default::default()
    });
    // Insert name and position component
    tile.insert(Name::new(name));
    tile.insert(*room_pos);

    // Add collider if needed
    if collider {
        tile.insert(Collider::Solid);
    };

    // Add door component or spawner components
    if tile_type == TileType::Door {
        tile.insert(Door);
    } else if tile_type == TileType::Spawner {
        let mut enemy_type: EnemyType = rand::random();

        if *room_type == RoomType::Boss {
            enemy_type = EnemyType::Boss;
        }

        tile.insert(Spawner { enemy_type });
    }
}
