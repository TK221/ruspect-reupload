use super::{enemy::behavior::EnemySlainEvent, player::DEFAULT_INVINCIBILITY_DURATION};
use crate::{
    map::map_generation::RoomPos,
    menu::{AppState, EndGameEvent},
    spawnable::{
        bullet::Bullet,
        enemy::{
            behavior::Enemy, behavior::Hitstun, enemy_types::spawn_enemy_type,
            enemy_types::EnemyResource, enemy_types::EnemyType,
        },
        movement::{
            DEFAULT_HITSTUN_DURATION, DEFAULT_KNOCKBACK_DURATION, DEFAULT_KNOCKBACK_STRENGTH,
        },
        player::Invincibility,
        player::Player,
        weapon::WeaponResource,
    },
    TIME_STEP,
};
use bevy::{core::FixedTimestep, prelude::*};
use serde::Deserialize;

pub struct SpawnablePlugin;

// --- Execute systems ---
impl Plugin for SpawnablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .label("step4")
                .with_run_criteria(FixedTimestep::step(TIME_STEP))
                .with_system(check_health)
                .after("step3"),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(check_spawnable_behavior.label("despawn").before("step1")),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame).with_system(check_damage.after("step3")),
        );
    }
}

// --- Components and Structs ---
#[derive(Component, Deserialize, Clone)]

/// Spawnable component that contains if they despawn and their despawn behavior
pub struct Spawnable {
    pub on_despawn: Vec<DespawnBehavior>,
    pub despawn: bool,
}

/// Enum for the different despawn behaviors
#[derive(Deserialize, Clone, PartialEq)]
pub enum DespawnBehavior {
    DieAtZero,
    SpawnNewMob(EnemyType),
    Despawn,
    EndGame,
}

/// Event for taking damage
pub struct TakeDamageEvent {
    pub entity: Entity,
    pub damage_entity: Entity,
}

/// Health component for enemies and player
#[derive(Component, Deserialize, Clone)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

// --- Methods for Structs ---
impl Health {
    /// Substracts damage from health
    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}

// --- System-Functions ---
/// Checks the health of spawnable entities and sets the despawn flag if the health is 0 or ends the game if the player-health is 0
fn check_health(
    mut health_query: Query<
        (
            &Health,
            &mut Spawnable,
            &mut Sprite,
            Option<&Player>,
            Option<&Enemy>,
        ),
        Or<(With<Player>, With<Enemy>)>,
    >,
    mut app_state: ResMut<State<AppState>>,
    mut ev_game_end: EventWriter<EndGameEvent>,
) {
    for (health, mut spawnable, mut sprite, player, enemy) in health_query.iter_mut() {
        if enemy.is_some() {
            sprite.color.set_a(health.health / health.max_health);
            if health.health <= 0.0 {
                spawnable.despawn = true;
            }
        } else if player.is_some() && health.health <= 0.0 {
            app_state.set(AppState::MainMenu).unwrap();
            ev_game_end.send(EndGameEvent {
                score: player.unwrap().score,
                boss_slain: false,
            });
        }
    }
}

/// Executes the despawn behavior of spawnable entities
fn check_spawnable_behavior(
    spawnables: Query<(
        &Spawnable,
        Entity,
        &Transform,
        Option<&RoomPos>,
        Option<&Health>,
    )>,
    mut player_query: Query<&mut Player, With<Player>>,
    enemy_query: Query<&Enemy, With<Enemy>>,
    mut commands: Commands,
    enemy_res: Res<EnemyResource>,
    weapon_res: Res<WeaponResource>,
    mut ev_enemy_slain: EventWriter<EnemySlainEvent>,
    mut ev_game_end: EventWriter<EndGameEvent>,
    mut app_state: ResMut<State<AppState>>,
) {
    for (spawnable, entity, transform, room_pos, health_option) in spawnables.iter() {
        let mut player = player_query.single_mut();

        let behaviors = spawnable.on_despawn.clone();
        let mut despawned_enemy = false;
        let mut spawned_enemy = false;
        if spawnable.despawn {
            if let Ok(enemy) = enemy_query.get(entity) {
                player.score += enemy.points
            }

            for behavior in behaviors {
                match behavior {
                    DespawnBehavior::DieAtZero => {
                        if let Some(health) = health_option {
                            if health.health <= 0.0 {
                                commands.entity(entity).despawn();
                                despawned_enemy = true;
                            }
                        }
                    }
                    DespawnBehavior::SpawnNewMob(mob_type) => {
                        if let Some(&room_pos) = room_pos {
                            spawn_enemy_type(
                                &mut commands,
                                &enemy_res,
                                &weapon_res,
                                &mob_type,
                                Vec2::new(transform.translation.x, transform.translation.y),
                                room_pos,
                            );
                            spawned_enemy = true;
                        }
                    }
                    DespawnBehavior::Despawn => {
                        commands.entity(entity).despawn();
                    }
                    DespawnBehavior::EndGame => {
                        ev_game_end.send(EndGameEvent {
                            score: player.score,
                            boss_slain: true,
                        });
                        app_state.set(AppState::MainMenu).unwrap();
                    }
                }
            }
        }
        if despawned_enemy && !spawned_enemy {
            if let Some(&room_pos) = room_pos {
                ev_enemy_slain.send(EnemySlainEvent(room_pos, entity));
            }
        }
    }
}

/// Iterates over the TakeDamageEvent and executes necessary actions
fn check_damage(
    mut take_damage: EventReader<TakeDamageEvent>,
    mut health_entity: Query<(
        Entity,
        &mut Health,
        &Transform,
        &Spawnable,
        Option<&Player>,
        Option<&Invincibility>,
        Option<&Enemy>,
    )>,
    mut damage_entity: Query<(&Transform, Option<&Bullet>, Option<&Enemy>)>,
    mut commands: Commands,
) {
    for take_damage in take_damage.iter() {
        if let Ok((
            entity,
            mut health,
            transform,
            spawnable,
            player_option,
            invincibility,
            enemy_option,
        )) = health_entity.get_mut(take_damage.entity)
        {
            if let Ok((damage_transform, damage_bullet_option, damage_enemy_option)) =
                damage_entity.get_mut(take_damage.damage_entity)
            {
                //Get damage and despawn if bullet
                let mut damage = 0.0;

                match (damage_bullet_option, damage_enemy_option) {
                    (Some(damage_bullet_option), None) => {
                        damage = damage_bullet_option.damage;
                    }
                    (None, Some(damage_enemy_option)) => {
                        damage = damage_enemy_option.damage;
                    }
                    _ => {}
                }

                // Insert invincibility if player is hit and hitstun for the enemy
                if player_option.is_some() {
                    if invincibility.is_none() {
                        health.take_damage(damage);
                        commands.entity(entity).insert(Invincibility {
                            duration: DEFAULT_INVINCIBILITY_DURATION,
                        });
                    }

                    if let Some(_enemy) = damage_enemy_option {
                        create_hitstun(
                            &mut commands,
                            take_damage.damage_entity,
                            damage_enemy_option,
                            damage_bullet_option,
                            spawnable,
                            Some((transform, damage_transform)),
                        );
                    }
                } else if enemy_option.is_some() {
                    health.take_damage(damage);
                    create_hitstun(
                        &mut commands,
                        entity,
                        enemy_option,
                        damage_bullet_option,
                        spawnable,
                        None,
                    );
                }
            }
        }
    }
}

/// Creates a hitstun effect for the enemy
fn create_hitstun(
    commands: &mut Commands,
    enemy_entity: Entity,
    enemy_option: Option<&Enemy>,
    bullet_option: Option<&Bullet>,
    spawnable: &Spawnable,
    transform_option: Option<(&Transform, &Transform)>,
) {
    if let Some(enemy) = enemy_option {
        let mut knockback_direction = Vec2::new(0.0, 0.0);
        let mut knockback_strength = DEFAULT_KNOCKBACK_STRENGTH;
        let mut knockback_duration = DEFAULT_KNOCKBACK_DURATION;
        let mut hitstun_duration = DEFAULT_HITSTUN_DURATION;

        if let Some((player_transform, enemy_transform)) = transform_option {
            knockback_direction = Vec2::new(
                enemy_transform.translation.x - player_transform.translation.x,
                enemy_transform.translation.y - player_transform.translation.y,
            )
            .normalize();
        }

        if let Some(bullet) = bullet_option {
            knockback_direction = bullet.direction.normalize();
            knockback_strength = bullet.knockback_strength;
            knockback_duration = bullet.knockback_duration;
            hitstun_duration = bullet.hitstun_duration;
        }

        let mut knockback = Vec2::new(0.0, 0.0);

        // Only apply knockback if enemy has more than 0 weight (i.e. not a boss)
        if enemy.weight > 0.0 {
            knockback = knockback_direction * knockback_strength / enemy.weight;
        }

        if !spawnable.despawn {
            commands.entity(enemy_entity).insert(Hitstun {
                knockback_direction: knockback,
                knockback_duration,
                hitstun_duration,
            });
        }
    }
}
