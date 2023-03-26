use crate::{
    menu::AppState,
    spawnable::{behavior::Health, enemy::enemy_types::Boss, player::Player, weapon::WeaponList},
};
use bevy::prelude::*;

pub struct HudPlugin;

// --- Execute systems ---
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::InGame).with_system(spawn_hud.label("step1")),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_main_hud)
                .with_system(show_boss_hud)
                .with_system(update_boss_hud)
                .with_system(update_weapon_hud),
        );
    }
}

// --- Components and Structs ---

/// The remaining health bar for the boss
#[derive(Component)]
struct BossHealth {}

/// The max health of the boss
#[derive(Component)]
struct BossMaxHealth {}

/// The remain reload time for the player
#[derive(Component)]
struct ReloadBar {}

/// The reamining ammo for the player
#[derive(Component)]
struct AmmoBar {}

/// The hud main text (health and score)
#[derive(Component)]
struct MainText {}

/// The name of the weapon
#[derive(Component)]
struct WeaponText {}

// --- System-Functions ---
/// Spawns the HUD with all the necessary components
fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(hud_wrapper())
        .with_children(|parent| {
            parent
                .spawn_bundle(main_hud_text(&asset_server))
                .insert(MainText {});
            parent
                .spawn_bundle(weapon_hud_text(&asset_server))
                .insert(WeaponText {});
            parent
                .spawn_bundle(boss_bar_max())
                .insert(BossMaxHealth {})
                .with_children(|parent| {
                    parent.spawn_bundle(boss_bar()).insert(BossHealth {});
                });
            parent
                .spawn_bundle(reload_bar_max())
                .with_children(|parent| {
                    parent.spawn_bundle(reload_bar()).insert(ReloadBar {});
                });
            parent.spawn_bundle(ammo_bar_max()).with_children(|parent| {
                parent.spawn_bundle(ammo_bar()).insert(AmmoBar {});
            });
        });
}

/// Makes the Boss-Health-Bar visible if a Boss is spawned
fn show_boss_hud(
    boss_spawn: Query<Entity, Added<Boss>>,
    mut query_hud: Query<&mut Visibility, Or<(With<BossHealth>, With<BossMaxHealth>)>>,
    // mut query_hud: Query<&mut Visibility, With<BossMaxHealth>>,
) {
    for _boss_spawn in boss_spawn.iter() {
        for mut boss_hud in query_hud.iter_mut() {
            boss_hud.is_visible = true;
        }
    }
}

/// Updates the main HUD with the current health and score
fn update_main_hud(
    mut query_player: Query<(&Player, &Health, &WeaponList)>,
    mut query_main_text: Query<&mut Text, (With<MainText>, Without<WeaponText>)>,
    mut query_weapon_text: Query<&mut Text, (With<WeaponText>, Without<MainText>)>,
) {
    let (player, health, weaponlist) = query_player.single_mut();
    let weapon = &weaponlist.weapons[0];

    let mut main_text = query_main_text.single_mut();
    main_text.sections[1].value = format!("{:?}", health.health);
    main_text.sections[3].value = format!("{:?}", player.score);

    let mut weapon_text = query_weapon_text.single_mut();
    weapon_text.sections[0].value = weapon.name.to_string();
}

// Updates the Boss-Health-Bar
fn update_boss_hud(
    query_boss_hud: Query<&Health, With<Boss>>,
    mut query_boss_health: Query<&mut Style, (With<BossHealth>, Without<BossMaxHealth>)>,
) {
    for health in query_boss_hud.iter() {
        for mut boss_health in query_boss_health.iter_mut() {
            boss_health.size = Size::new(
                Val::Percent(health.health / health.max_health * 100.0),
                Val::Percent(100.0),
            );
        }
    }
}

// Updates the Reload-Bar and the Ammo-Bar
fn update_weapon_hud(
    mut query_player: Query<&WeaponList, With<Player>>,
    mut query_reload_bar: Query<&mut Style, (With<ReloadBar>, Without<AmmoBar>)>,
    mut query_ammo_bar: Query<&mut Style, (With<AmmoBar>, Without<ReloadBar>)>,
) {
    let weaponlist = query_player.single_mut();
    let mut reload_bar_style = query_reload_bar.single_mut();
    let mut ammo_bar_style = query_ammo_bar.single_mut();

    // Gets the current weapon
    let weapon = &weaponlist.weapons[0];

    if weapon.reload_time.current > 0.0 {
        reload_bar_style.size = Size::new(
            Val::Percent(100.0),
            Val::Percent(weapon.reload_time.current / weapon.reload_time.max * 100.0),
        );
    } else {
        reload_bar_style.size = Size::new(Val::Percent(0.0), Val::Percent(0.0));
        ammo_bar_style.size = Size::new(Val::Percent(100.0), Val::Percent(100.0));
    }

    if weapon.clip_size.current != weapon.clip_size.max {
        ammo_bar_style.size = Size::new(
            Val::Percent(100.0),
            Val::Percent(100.0 - weapon.clip_size.current / weapon.clip_size.max * 100.0),
        );
    } else if weapon.reload_time.current > 0.0 {
        ammo_bar_style.size = Size::new(Val::Percent(100.0), Val::Percent(100.0));
    } else {
        ammo_bar_style.size = Size::new(Val::Percent(0.0), Val::Percent(0.0));
    }
}

// --- Ui-Elements ---

/// Wrapper for the whole HUD
fn hud_wrapper() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}

/// Health and score of the player
fn main_hud_text(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Lives: ".to_string(),
                    style: main_hud_text_style(asset_server),
                },
                TextSection {
                    value: "".to_string(),
                    style: main_hud_text_style(asset_server),
                },
                TextSection {
                    value: "\nScore: ".to_string(),
                    style: main_hud_text_style(asset_server),
                },
                TextSection {
                    value: "".to_string(),
                    style: main_hud_text_style(asset_server),
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Weapon hud text
fn weapon_hud_text(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/orangeJuice2.0.ttf"),
                    font_size: 30.0,
                    color: Color::rgb(1.0, 0.5, 0.5),
                },
            }],
            alignment: TextAlignment {
                vertical: (VerticalAlign::Center),
                horizontal: (HorizontalAlign::Right),
            },
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                ..Default::default()
            },
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Full health bar for the boss
fn boss_bar_max() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(70.0), Val::Percent(2.0)),
            position_type: PositionType::Relative,
            position: Rect {
                bottom: Val::Percent(-25.0),
                ..Default::default()
            },
            ..Default::default()
        },
        visibility: Visibility { is_visible: false },
        color: UiColor(Color::rgb(1.0, 1.0, 1.0)),
        ..Default::default()
    }
}

/// Remaining health bar of the boss
fn boss_bar() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Relative,
            align_self: AlignSelf::FlexStart,
            ..Default::default()
        },
        visibility: Visibility { is_visible: false },
        color: UiColor(Color::rgb(1.0, 0.0, 0.0)),
        ..Default::default()
    }
}

/// Full reload time bar
fn reload_bar_max() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(15.0), Val::Px(100.0)),
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(50.0),
                right: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.0, 0.8, 0.0)),
        ..Default::default()
    }
}

/// Remaining reload time bar
fn reload_bar() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Relative,
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.2, 0.0, 0.0)),
        ..Default::default()
    }
}

/// Full ammo bar
fn ammo_bar_max() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(15.0), Val::Px(100.0)),
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(50.0),
                right: Val::Px(35.0),
                ..Default::default()
            },
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.8, 0.8, 0.0)),
        ..Default::default()
    }
}

/// Remaining ammo bar
fn ammo_bar() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Relative,
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.0, 0.0, 0.0)),
        ..Default::default()
    }
}

/// Style for the main hud text
fn main_hud_text_style(asset_server: &Res<AssetServer>) -> TextStyle {
    TextStyle {
        font: asset_server.load("fonts/orangeJuice2.0.ttf"),
        font_size: 40.0,
        color: Color::rgb(1.0, 0.5, 0.5),
    }
}
