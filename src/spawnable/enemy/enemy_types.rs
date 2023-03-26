use std::collections::HashMap;

use bevy::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};
use serde::Deserialize;

use crate::{
    map::map_generation::RoomPos,
    spawnable::{
        behavior::{Health, Spawnable},
        movement::Collider,
        weapon::{Weapon, WeaponList, WeaponResource, WeaponTypes},
    },
};

use super::behavior::Enemy;

/// Boss-Component as a marker for the boss
#[derive(Component)]
pub struct Boss {}

/// Enemy-Resource for loading enemys from ron file
#[derive(Deserialize)]
pub struct EnemyResource {
    enemys: HashMap<EnemyType, EnemyRon>,
}

/// Struct for the enemy-resource, containing the enemy itself, the health and the behavior
#[derive(Deserialize, Clone)]
pub struct EnemyRon {
    data: Enemy,
    health: Health,
    behavior: Spawnable,
}

/// Enemy-Type as a marker for the enemy
#[derive(Component, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub enum EnemyType {
    BigBlob,
    MediumBlob,
    SmallBlob,
    PistolEnemy,
    SplitShotEnemy,
    MachineGunEnemy,
    ShotgunEnemy,
    SniperEnemy,
    CrossEnemy,
    CircleEnemy,
    Boss,
}

impl Distribution<EnemyType> for Standard {
    /// Get a random enemytype (excluding the boss)
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyType {
        match rng.gen_range(0..=7) {
            0 => EnemyType::BigBlob,
            1 => EnemyType::PistolEnemy,
            2 => EnemyType::SplitShotEnemy,
            3 => EnemyType::ShotgunEnemy,
            4 => EnemyType::SniperEnemy,
            5 => EnemyType::CrossEnemy,
            6 => EnemyType::CircleEnemy,
            _ => EnemyType::MachineGunEnemy,
        }
    }
}

// --- Functions ---
/// Spawns an enemy with a given type by using the enemy-resource
pub fn spawn_enemy_type(
    commands: &mut Commands,
    enemy_res: &EnemyResource,
    weapon_res: &WeaponResource,
    enemy_type: &EnemyType,
    position: Vec2,
    room_pos: RoomPos,
) {
    let enemy = &enemy_res.enemys[enemy_type];
    let mut entity = commands.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: position.extend(0.0),
            scale: enemy.data.scale,
            ..Default::default()
        },
        sprite: Sprite {
            color: enemy.data.color,
            ..Default::default()
        },
        ..Default::default()
    });

    entity
        .insert(enemy.data.clone())
        .insert(enemy.health.clone())
        .insert(enemy.behavior.clone())
        .insert(enemy_type.clone())
        .insert(Collider::Enemy)
        .insert(room_pos);

    let mut weapon_list: Vec<Weapon> = Vec::new();
    for weapon in &enemy.data.weapon {
        if weapon != &WeaponTypes::None {
            let weapon = &weapon_res.weapons[weapon];
            weapon_list.push(weapon.clone());
        }
    }

    if !weapon_list.is_empty() {
        entity.insert(WeaponList {
            weapons: weapon_list,
        });
    }

    if enemy.data.is_boss {
        entity.insert(Boss {});
    }
}
