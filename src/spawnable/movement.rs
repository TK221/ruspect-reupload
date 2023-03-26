use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{
    map::{
        map_generation::RoomPos,
        room::{LeaveRoomEvent, TransitionDirection},
    },
    menu::AppState,
    spawnable::behavior::{Spawnable, TakeDamageEvent},
    TIME_STEP,
};

// --- Constants ---
pub const DEFAULT_HITSTUN_DURATION: f32 = 8.0;
pub const DEFAULT_KNOCKBACK_DURATION: f32 = 5.0;
pub const DEFAULT_KNOCKBACK_STRENGTH: f32 = 100.0;

pub struct MovementPlugin;

// --- Execute Systems ---
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>()
            .add_event::<MoveEntity>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("step2")
                    .with_system(movement_and_collision)
                    .after("step1"),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .label("step3")
                    .with_system(execute_movement)
                    .after("step2"),
            );
    }
}

// --- Components ---
/// Movement component for moving entities
#[derive(Component)]
pub struct Movement {
    pub direction: Vec2,
    pub transform: Transform,
    pub speed: f32,
}

/// Event for moving entities
pub struct MoveEntity {}

/// Collider component for entities, defines collider-type
#[derive(Component, Clone, Copy)]
pub enum Collider {
    Solid,
    Enemy,
    Player,
    EnemyBullet,
    PlayerBullet,
    RoomTransition,
}

// --- System-Functions ---
/// Moves entities theoretically and checks for collisions to resolve them
fn movement_and_collision(
    time: Res<Time>,
    mut movement_query: Query<
        (
            &mut Movement,
            &Transform,
            &Collider,
            Option<&mut Spawnable>,
            Entity,
        ),
        With<Movement>,
    >,
    mut take_damage: EventWriter<TakeDamageEvent>,
    mut move_entity: EventWriter<MoveEntity>,
    collider_query: Query<(&Collider, &Transform, Entity)>,
    transition_query: Query<(&RoomPos, &TransitionDirection)>,
    mut ev_leave_room: EventWriter<LeaveRoomEvent>,
) {
    let mut max_movement = 0.0;

    for (mut movement, transform, _collider, _spawnable_option, _entity) in
        movement_query.iter_mut()
    {
        // normalize direction vector
        let normalized_direction = movement.direction;

        movement.direction = normalized_direction;

        max_movement = f32::max(max_movement, movement.speed);

        movement.transform = *transform;
    }

    //Get the time step for adjusting speed based on time
    let time_step = time.delta_seconds() / TIME_STEP as f32;

    for counter in 0..((max_movement.ceil() * 2.0) as i32) {
        for (mut movement, _transform, collider, spawnable_option, entity) in
            movement_query.iter_mut()
        {
            if movement.direction.length() > 0.0 && movement.speed > 0.0 {
                let mut step_movement = Vec3::new(0.0, 0.0, 0.0);

                if counter % 2 == 0 && movement.direction.x.abs() > 0.0 {
                    step_movement.x += movement.direction.x * time_step;
                } else if counter % 2 != 0 && movement.direction.y.abs() > 0.0 {
                    step_movement.y += movement.direction.y * time_step;
                }

                movement.speed -= 1.0;
                //Add the calculated step movement to the transform
                movement.transform.translation += step_movement;

                for (collider_type, collider_transform, collider_entity) in collider_query.iter() {
                    // check if entity is colliding with another entity
                    if check_collision(&movement.transform, collider_transform)
                        && collider_entity != entity
                    {
                        //Check Collision between all different collider types
                        match (collider, collider_type) {
                            (Collider::Player | Collider::Enemy, Collider::Solid) => {}

                            (Collider::Enemy, Collider::Enemy) => {}

                            (Collider::Player, Collider::RoomTransition) => {
                                if let Ok((room_pos, direction)) =
                                    transition_query.get(collider_entity)
                                {
                                    ev_leave_room.send(LeaveRoomEvent(*room_pos, *direction));
                                }
                            }

                            (Collider::Player, Collider::Enemy) => {
                                movement.transform.translation -= step_movement;
                                take_damage.send(TakeDamageEvent {
                                    entity,
                                    damage_entity: collider_entity,
                                });
                            }

                            (Collider::Enemy, Collider::Player) => {
                                movement.transform.translation -= step_movement;
                                take_damage.send(TakeDamageEvent {
                                    entity: collider_entity,
                                    damage_entity: entity,
                                });
                            }

                            (
                                Collider::EnemyBullet | Collider::PlayerBullet,
                                Collider::Solid | Collider::RoomTransition,
                            ) => {
                                if let Some(mut spawnable) = spawnable_option {
                                    spawnable.despawn = true;
                                }
                                break;
                            }

                            (Collider::EnemyBullet, Collider::Player)
                            | (Collider::Player, Collider::EnemyBullet) => {
                                take_damage.send(TakeDamageEvent {
                                    entity: collider_entity,
                                    damage_entity: entity,
                                });
                                if let Some(mut spawnable) = spawnable_option {
                                    spawnable.despawn = true;
                                }
                            }

                            (Collider::PlayerBullet, Collider::Enemy) => {
                                take_damage.send(TakeDamageEvent {
                                    entity: collider_entity,
                                    damage_entity: entity,
                                });
                                if let Some(mut spawnable) = spawnable_option {
                                    spawnable.despawn = true;
                                }
                            }

                            (Collider::Enemy, Collider::PlayerBullet) => {
                                take_damage.send(TakeDamageEvent {
                                    entity,
                                    damage_entity: collider_entity,
                                });
                            }

                            _ => {
                                break;
                            }
                        }

                        if counter % 2 == 0 {
                            movement.direction.x = 0.0;
                        } else {
                            movement.direction.y = 0.0;
                        }
                        movement.transform.translation -= step_movement;
                        break;
                    }
                }
            }
        }
    }

    //Send event to move entity
    move_entity.send(MoveEntity {});
}

// --- Functions ---
/// Executes the movement by updating the transform of the entity
fn execute_movement(
    mut commands: Commands,
    mut movement_query: Query<(&Movement, &mut Transform, Entity), With<Movement>>,
    mut move_entity: EventReader<MoveEntity>,
) {
    for _move_entity in move_entity.iter() {
        for (movement, mut transform, entity) in movement_query.iter_mut() {
            transform.translation = movement.transform.translation;
            commands.entity(entity).remove::<Movement>();
        }
    }
}

/// Returns true if object 1 collides with object 2 (only works for AABB)
pub fn check_collision(object_1: &Transform, object_2: &Transform) -> bool {
    collide(
        object_1.translation,
        object_1.scale.truncate(),
        object_2.translation,
        object_2.scale.truncate(),
    )
    .is_some()
}
