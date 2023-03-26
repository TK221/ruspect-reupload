use crate::{
    map::{
        calc_mid_room_pos,
        room::{LeaveRoomEvent, TransitionDirection},
        TILE_SIZE, X_MAP_LENGTH, X_ROOM_LENGTH, Y_MAP_LENGTH, Y_ROOM_LENGTH,
    },
    menu::AppState,
    spawnable::{
        behavior::Health,
        behavior::Spawnable,
        behavior::TakeDamageEvent,
        movement::{Collider, MoveEntity, Movement},
        weapon::{Weapon, WeaponList, WeaponResource, WeaponTypes},
    },
    BLINKING_SPEED_PLAYER,
};
use bevy::prelude::*;

// --- Plugin ---
pub struct PlayerPlugin;

// --- Constants ---
pub const PLAYER_HEALTH: f32 = 5.0;
pub const PLAYER_SPEED: f32 = 20.0;
pub const DEFAULT_INVINCIBILITY_DURATION: f32 = 240.0;
pub const PLAYER_COLOR: Color = Color::rgb(0.0, 0.0, 1.0);

// --- Components ---
/// Player component
#[derive(Component)]
pub struct Player {
    pub score: i32,
    pub speed: f32,
    pub color: Color,
}

/// Invincibility component for the player
#[derive(Component)]
pub struct Invincibility {
    pub duration: f32,
}

// --- Execute systems ---
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>()
            .add_event::<MoveEntity>()
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(spawn_player))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("step1")
                    .with_system(player_shooting)
                    .with_system(player_movement_input)
                    .with_system(room_transmit),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(check_invincibility.after("step3")),
            );
    }
}

// --- System-Functions ---
/// Spawns the player at the center of the map
fn spawn_player(mut commands: Commands, weapon_res: Res<WeaponResource>) {
    let random_weapon: WeaponTypes = rand::random();
    let weaponlist: Vec<Weapon> = vec![weapon_res.weapons[&random_weapon].clone()];

    let curr_room_pos = calc_mid_room_pos(X_MAP_LENGTH / 2, Y_MAP_LENGTH / 2);

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(curr_room_pos.x, curr_room_pos.y, 4.0),
                scale: Vec3::new(25.0, 25.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player {
            score: 0,
            speed: PLAYER_SPEED,
            color: PLAYER_COLOR,
        })
        .insert(Collider::Player)
        .insert(Health {
            health: PLAYER_HEALTH,
            max_health: PLAYER_HEALTH,
        })
        .insert(WeaponList {
            weapons: weaponlist,
        })
        .insert(Spawnable {
            on_despawn: vec![],
            despawn: false,
        });
}

/// Shoots a weapon with the WASD keys
fn player_shooting(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&Transform, &mut WeaponList), With<Player>>,
    mut time: Res<Time>,
) {
    let (transform, mut weaponlist) = player_query.single_mut();
    let mut dir: Vec2 = Vec2::new(0.0, 0.0);

    if keys.any_pressed([KeyCode::W, KeyCode::S]) {
        dir.y += (keys.pressed(KeyCode::W) as i32 - keys.pressed(KeyCode::S) as i32) as f32;
    } else if keys.any_pressed([KeyCode::A, KeyCode::D]) {
        dir.x += (keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32) as f32;
    }

    weaponlist.weapons[0].shoot_weapon(
        &mut commands,
        &mut time,
        dir,
        Vec2::new(transform.translation.x, transform.translation.y),
        true,
    )
}

/// Moves the player with Arrow keys
fn player_movement_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&Player, &Transform, Entity)>,
) {
    let (player, transform, player_entity) = player_query.single_mut();
    let mut dir: Vec2 = Vec2::new(0.0, 0.0);

    // check direction input
    dir.y += (keys.pressed(KeyCode::Up) as i32 - keys.pressed(KeyCode::Down) as i32) as f32;
    dir.x += (keys.pressed(KeyCode::Right) as i32 - keys.pressed(KeyCode::Left) as i32) as f32;

    if dir.length() != 0.0 {
        commands.entity(player_entity).insert(Movement {
            direction: dir,
            transform: *transform,
            speed: player.speed,
        });
    }
}

/// Moves the player to the next room
fn room_transmit(
    mut ev_leave_room: EventReader<LeaveRoomEvent>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    for ev_leave_room in ev_leave_room.iter() {
        let mut transform = player_query.single_mut();
        let mut curr_room_pos = calc_mid_room_pos(ev_leave_room.0.x, ev_leave_room.0.y);

        match ev_leave_room.1 {
            TransitionDirection::Up => {
                curr_room_pos.y -= (Y_ROOM_LENGTH / 2 - 1) as f32 * TILE_SIZE;
            }
            TransitionDirection::Down => {
                curr_room_pos.y += (Y_ROOM_LENGTH / 2 - 1) as f32 * TILE_SIZE;
            }
            TransitionDirection::Left => {
                curr_room_pos.x += (X_ROOM_LENGTH / 2 - 1) as f32 * TILE_SIZE;
            }
            TransitionDirection::Right => {
                curr_room_pos.x -= (X_ROOM_LENGTH / 2 - 1) as f32 * TILE_SIZE;
            }
        }

        transform.translation = Vec3::new(curr_room_pos.x, curr_room_pos.y, 100.0);
    }
}

/// Checks if the player is invincible, changes the color of the player if he is and removes the invincibility after the timer is over
fn check_invincibility(
    mut commands: Commands,
    mut query: Query<(&Player, &mut Invincibility, &mut Sprite, Entity), With<Player>>,
) {
    for (player, mut invincibility, mut sprite, entity) in &mut query.iter_mut() {
        if invincibility.duration > 0.0 {
            invincibility.duration -= 1.0;

            sprite.color = if invincibility.duration % BLINKING_SPEED_PLAYER
                > BLINKING_SPEED_PLAYER / 2.0 - 1.0
            {
                Color::rgb(1.0, 1.0, 1.0)
            } else {
                player.color
            };

            sprite.color.set_a(0.2);
        } else {
            sprite.color = player.color;

            commands.entity(entity).remove::<Invincibility>();
        }
    }
}
