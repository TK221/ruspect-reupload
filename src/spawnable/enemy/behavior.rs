use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::{
    map::map_generation::RoomPos,
    menu::AppState,
    spawnable::{
        behavior::{Health, Spawnable},
        enemy::enemy_types::Boss,
        movement::Movement,
        player::Player,
        weapon::{WeaponList, WeaponTypes},
    },
};

pub struct EnemyPlugin;

// --- Execute systems ---
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemySlainEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame).with_system(enemy_behaviour.before("step2")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(check_hitstun)
                    .before("step4"),
            );
    }
}

// --- Components and Structs ---
/// Hitstun (and knockback) component for the enemy
#[derive(Component, Clone)]
pub struct Hitstun {
    pub knockback_direction: Vec2,
    pub knockback_duration: f32,
    pub hitstun_duration: f32,
}
/// Enemy component
#[derive(Component, Deserialize, Clone)]
pub struct Enemy {
    pub speed: f32,
    pub color: Color,
    pub points: i32,
    pub scale: Vec3,
    pub damage: f32,
    pub weight: f32,
    pub behavior: Vec<EnemyBehavior>,
    pub weapon: Vec<WeaponTypes>,
    pub is_boss: bool,
}

/// Enemy behavior for the enemy
#[derive(Deserialize, Clone)]
pub enum EnemyBehavior {
    MoveToPlayer(f32),
    ShootAtPlayer,
}

pub struct EnemySlainEvent(pub RoomPos, pub Entity);

// --- System-Functions ---
/// Execeutes enemy behavior
fn enemy_behaviour(
    mut commands: Commands,
    mut enemy_query: Query<
        (&Enemy, &Transform, &Spawnable, Entity),
        (
            With<Enemy>,
            Without<Player>,
            Or<(Without<Hitstun>, With<Boss>)>,
        ),
    >,
    mut weapon_list_query: Query<(&mut WeaponList, Entity), With<Enemy>>,
    mut player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut time: Res<Time>,
) {
    for (enemy, enemy_transform, spawnable, entity) in enemy_query.iter_mut() {
        for behavior in enemy.behavior.iter() {
            match behavior {
                EnemyBehavior::MoveToPlayer(range) => {
                    let player_transform = player_query.single_mut();

                    let direction = player_transform.translation - enemy_transform.translation;

                    if direction.length() > *range && !spawnable.despawn {
                        commands.entity(entity).insert(Movement {
                            direction: Vec2::new(direction.x, direction.y).normalize(),
                            transform: *enemy_transform,
                            speed: enemy.speed,
                        });
                    }
                }
                EnemyBehavior::ShootAtPlayer => {
                    let player_transform = player_query.single_mut();

                    let direction = Vec2::new(
                        player_transform.translation.x - enemy_transform.translation.x,
                        player_transform.translation.y - enemy_transform.translation.y,
                    )
                    .normalize();

                    for (mut weapon_list, weapon_entity) in weapon_list_query.iter_mut() {
                        if weapon_entity == entity {
                            //Shoot random weapon
                            let weapon = &mut weapon_list.weapons
                                [rand::thread_rng().gen_range(0..enemy.weapon.len())];

                            weapon.shoot_weapon(
                                &mut commands,
                                &mut time,
                                direction,
                                Vec2::new(
                                    enemy_transform.translation.x,
                                    enemy_transform.translation.y,
                                ),
                                false,
                            )
                        }
                    }
                }
            }
        }
    }
}

/// Checks if enemy is hitstunned and if so, moves enemy
fn check_hitstun(
    mut hitstun_query: Query<
        (
            &mut Hitstun,
            Entity,
            &mut Sprite,
            &Enemy,
            &Health,
            &Transform,
            &Spawnable,
        ),
        (With<Hitstun>, With<Enemy>),
    >,
    mut commands: Commands,
) {
    for (mut hitstun, entity, mut sprite, enemy, _health, transform, spawnable) in
        hitstun_query.iter_mut()
    {
        if hitstun.hitstun_duration > 0.0 && !spawnable.despawn {
            hitstun.hitstun_duration -= 1.0;
            sprite.color = Color::rgb(1.0, 1.0, 1.0);

            // calculate knockback
            if hitstun.knockback_duration > 0.0 {
                commands.entity(entity).insert(Movement {
                    direction: hitstun.knockback_direction.normalize(),
                    transform: *transform,
                    speed: hitstun.knockback_direction.length(),
                });

                hitstun.knockback_duration -= 1.0;
            }
        } else {
            sprite.color = enemy.color;
            commands.entity(entity).remove::<Hitstun>();
        }
    }
}
