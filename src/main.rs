#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use ruspect::GamePlugin;

// --- Plugin-Declaration and runs Game---
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(WindowDescriptor {
            title: "Ruspect".to_string(),
            width: 1500.,
            height: 900.,
            resizable: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .run();
}
