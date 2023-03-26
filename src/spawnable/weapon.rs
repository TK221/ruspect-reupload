use std::collections::HashMap;

use crate::{
    menu::AppState,
    spawnable::{
        behavior::DespawnBehavior, behavior::Spawnable, bullet::Bullet, movement::Collider,
    },
};

use bevy::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, thread_rng, Rng};
use serde::Deserialize;

pub struct WeaponPlugin;

// --- Execute systems ---
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(reload_weapon)
                .before("step2"),
        );
    }
}

// --- Components and Structs ---
/// Weapon-Resource for loading weapons from ron file
#[derive(Deserialize, Clone)]
pub struct WeaponResource {
    pub weapons: HashMap<WeaponTypes, Weapon>,
}

/// Maximum and current values for reload, ammo and magazine
#[derive(Deserialize, Clone)]
pub struct MaxCurrent {
    pub max: f32,
    pub current: f32,
}

/// List of weapons
#[derive(Deserialize, Component, Clone)]
pub struct WeaponList {
    pub weapons: Vec<Weapon>,
}

/// Weapon-Component
#[derive(Deserialize, Component, Clone)]
pub struct Weapon {
    pub name: String,
    pub clip_size: MaxCurrent,
    pub fire_rate: MaxCurrent,
    pub reload_time: MaxCurrent,
    pub speed: f32,
    pub damage: f32,
    pub knockback_strength: f32,
    pub knockback_duration: f32,
    pub hitstun_duration: f32,
    pub spread: f32,
    pub shooting_pattern: Vec<f32>,
}

/// Enum for weapon types (values are defined in the ron file)
#[derive(Component, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WeaponTypes {
    None,
    Pistol,
    Sniper,
    Shotgun,
    MachineGun,
    PlayerPistol,
    PlayerSniper,
    PlayerShotgun,
    PlayerMachineGun,
    SplitShot,
    CrossGun,
    CircleGun,
}

// --- Methods for Structs ---
impl Distribution<WeaponTypes> for Standard {
    /// Returns a random player-weapon for the player
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WeaponTypes {
        match rng.gen_range(0..=3) {
            0 => WeaponTypes::PlayerPistol,
            1 => WeaponTypes::PlayerSniper,
            2 => WeaponTypes::PlayerShotgun,
            _ => WeaponTypes::PlayerMachineGun,
        }
    }
}

impl Weapon {
    /// Spawns a bullet entity with the given parameters
    pub fn spawn_bullet(
        &self,
        commands: &mut Commands,
        direction: Vec2,
        start_x: f32,
        start_y: f32,
        is_player: bool,
    ) {
        // Get the color of the bullet depending on if it is a player bullet or not
        let (bullet_color, bullet_type) = if is_player {
            (Color::rgb(0.8, 0.8, 0.0), Collider::PlayerBullet)
        } else {
            (Color::rgb(0.8, 0.0, 0.0), Collider::EnemyBullet)
        };

        // Iterate through the shooting patterns and spawn a bullet for each one
        for angle in self.shooting_pattern.iter() {
            // Calculate the bullet's angle
            let mut rng = thread_rng();
            let mut angle_modifier = *angle;
            if self.spread != 0.0 {
                angle_modifier += rng.gen_range(-self.spread..self.spread);
            }

            let direction_angle =
                ((direction.x.abs() / direction.y).atan() + angle_modifier) % core::f32::consts::PI;

            let mut modifier = 1.0;

            if (direction_angle - angle_modifier).signum() != direction_angle.signum() {
                modifier = -modifier;
            }

            let bullet_direction;
            if direction.x != 0.0 {
                bullet_direction =
                    Vec2::new(1.0 * direction.x.signum(), 1.0 / direction_angle.tan()) * modifier;
            } else if direction_angle == 0.0 {
                bullet_direction = Vec2::new(0.0, direction.y.signum());
            } else {
                bullet_direction = Vec2::new(
                    1.0 * direction.x.signum(),
                    1.0 / direction_angle.tan() * direction.y.signum(),
                ) * modifier;
            }

            // Spawn the bullet
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(start_x, start_y, 0.0),
                        scale: Vec3::new(10.0, 10.0, 0.0),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        color: bullet_color,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Bullet {
                    speed: self.speed,
                    direction: bullet_direction,
                    damage: self.damage,
                    is_player,
                    knockback_strength: self.knockback_strength,
                    knockback_duration: self.knockback_duration,
                    hitstun_duration: self.hitstun_duration,
                })
                .insert(bullet_type)
                .insert(Spawnable {
                    on_despawn: [DespawnBehavior::Despawn].to_vec(),
                    despawn: false,
                });
        }
    }

    /// Checks if the weapon is ready to fire and if so, spawns a bullet
    pub fn shoot_weapon(
        &mut self,
        commands: &mut Commands,
        time: &mut Res<Time>,
        direction: Vec2,
        start: Vec2,
        is_player: bool,
    ) {
        if self.reload_time.current <= 0.0 {
            self.fire_rate.current -= time.delta_seconds();

            if direction.length() > 0.0 && self.fire_rate.current <= 0.0 {
                if self.clip_size.current > 0.0 || self.clip_size.max == 0.0 {
                    self.spawn_bullet(commands, direction, start.x, start.y, is_player);

                    if self.clip_size.max > 0.0 {
                        self.clip_size.current -= 1.0;

                        if self.clip_size.current == 0.0 {
                            self.reload_time.current = self.reload_time.max;
                            self.clip_size.current = self.clip_size.max;
                        }
                    }

                    self.fire_rate.current = self.fire_rate.max;
                } else {
                    self.reload_time.current = self.reload_time.max;
                    self.clip_size.current = self.clip_size.max;
                }
            }
        }
    }
}

// --- System-Functions ---
/// Automatically reloads the weapon if it empty
fn reload_weapon(mut weapon_query: Query<&mut WeaponList>, time: Res<Time>) {
    for mut weapon_list in weapon_query.iter_mut() {
        for weapon in weapon_list.weapons.iter_mut() {
            if weapon.reload_time.current > 0.0 {
                weapon.reload_time.current -= time.delta_seconds();
            }
        }
    }
}
