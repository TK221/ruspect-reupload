// --- Plugins ---
pub mod camera;
pub mod debug;
pub mod hud;
pub mod map;
pub mod menu;
pub mod spawnable;

use camera::CameraPlugin;
use debug::DebugPlugin;
use hud::HudPlugin;
use map::MapPlugin;
use menu::{AppState, MenuPlugin};
use spawnable::behavior::SpawnablePlugin;
use spawnable::bullet::BulletPlugin;
use spawnable::enemy::behavior::EnemyPlugin;
use spawnable::enemy::enemy_types::EnemyResource;
use spawnable::movement::MovementPlugin;
use spawnable::player::PlayerPlugin;
use spawnable::weapon::{WeaponPlugin, WeaponResource};

// --- Imports ---
use bevy::app::App;
use bevy::prelude::*;
use ron::de::from_bytes;
use std::process::exit;

// --- Global Constants ---
pub const TIME_STEP: f64 = 1.0 / 60.0;
const BLINKING_SPEED_PLAYER: f32 = 16.0;

// --- Components and Structs ---
#[derive(Component)]
pub struct PlayerCamera;

// --- Plugin declaration ---
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerPlugin)
            .add_state(AppState::MainMenu)
            .add_plugin(BulletPlugin)
            .add_plugin(EnemyPlugin)
            .add_plugin(DebugPlugin)
            .add_plugin(MapPlugin)
            .add_plugin(SpawnablePlugin)
            .add_plugin(HudPlugin)
            .add_plugin(WeaponPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(MovementPlugin)
            .add_plugin(MenuPlugin);

        // Load files
        let enemy_resource =
            from_bytes::<EnemyResource>(include_bytes!("../assets/resources/enemy.ron"));
        let weapon_resource =
            from_bytes::<WeaponResource>(include_bytes!("../assets/resources/weapon.ron"));

        // Checks if resource-files are corectly loaded
        match (enemy_resource, weapon_resource) {
            (Ok(enemy_resource), Ok(weapon_resource)) => {
                app.insert_resource(enemy_resource);
                app.insert_resource(weapon_resource);
            }
            (enemy_resource, weapon_resource) => {
                if let Err(err) = enemy_resource {
                    println!("Error loading enemy resource: {}", err);
                }
                if let Err(err) = weapon_resource {
                    println!("Error loading weapon resource: {}", err);
                }
                println!("Error loading resource(s) - Ending program");
                exit(1);
            }
        }
    }
}
