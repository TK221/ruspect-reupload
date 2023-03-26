use bevy::{app::AppExit, prelude::*};

pub struct MenuPlugin;

// --- Execute Systems ---
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::MainMenu)
                .with_system(cleanup)
                .with_system(spawn_menu),
        )
        .add_event::<EndGameEvent>()
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(cleanup))
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(main_menu_controls))
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(menu_on_escape));
    }
}

// --- Components and Structs ---
/// Represents the current state of the game (main menu or in game)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
}

/// Represents the state of the menu (represented by the current selection)
#[derive(Component)]
pub struct MenuState {
    state: i32,
    number_of_states: i32,
}

/// Marks the Menu-Options for querying
#[derive(Component)]
pub struct MenuOptions {}

/// Event for showing the score at the end of the game
pub struct EndGameEvent {
    pub score: i32,
    pub boss_slain: bool,
}

// --- System-Functions ---
/// Goes back to the main menu on escape
fn menu_on_escape(mut keys: ResMut<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        keys.reset(KeyCode::Escape);
        app_state.set(AppState::MainMenu).unwrap();
    }
}

/// Controls the main menu
fn main_menu_controls(
    keys: ResMut<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
    mut query_menu: Query<&mut MenuState>,
    mut query_menu_text: Query<&mut Text, With<MenuOptions>>,
    mut exit: EventWriter<AppExit>,
) {
    if let Ok(mut menu_state) = query_menu.get_single_mut() {
        let mut menu_text = query_menu_text.single_mut();

        //Adjust Menu State based on keys
        if keys.any_just_pressed([KeyCode::Up, KeyCode::W]) {
            menu_state.state += 1;
        } else if keys.any_just_pressed([KeyCode::Down, KeyCode::S]) {
            menu_state.state -= 1;
        }

        //Wrap around if end of list is reached
        if menu_state.state < 0 {
            menu_state.state = menu_state.number_of_states - 1;
        } else if menu_state.state > menu_state.number_of_states - 1 {
            menu_state.state = 0;
        }

        //Change the color for the text of the menu
        for (i, text) in menu_text.sections.iter_mut().enumerate() {
            if i == menu_state.state as usize {
                text.style.color = Color::RED;
            } else {
                text.style.color = Color::GRAY;
            }
        }

        // Exit the game on escape
        if keys.just_pressed(KeyCode::Escape) {
            exit.send(AppExit);
        }

        //Start or exit the game on enter, depending on the menu state
        if keys.just_pressed(KeyCode::Return) {
            match menu_state.state {
                0 => {
                    println!("Start game");
                    app_state.set(AppState::InGame).unwrap();
                }
                1 => {
                    exit.send(AppExit);
                }
                _ => {
                    println!("Error");
                }
            }
        }
    }
}

/// Spawns the menu
fn spawn_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_game_end: EventReader<EndGameEvent>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(menu_wrapper())
        .with_children(|parent| {
            parent.spawn_bundle(header(&asset_server));
        })
        .with_children(|parent| {
            parent.spawn_bundle(logo(&asset_server));
        })
        .with_children(|parent| {
            parent.spawn_bundle(spacer());
        })
        .with_children(|parent| {
            for game_end in ev_game_end.iter() {
                parent.spawn_bundle(menu_text_after_game_end(
                    &asset_server,
                    game_end.score,
                    game_end.boss_slain,
                ));
                parent.spawn_bundle(spacer());
            }
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(menu_options_text(&asset_server))
                .insert(MenuOptions {});
        })
        .insert(MenuState {
            state: 0,
            number_of_states: 2,
        });
}

/// Despawns all entitys
fn cleanup(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// --- Ui-Elements ---
/// Wrapper for the menu
fn menu_wrapper() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            align_content: AlignContent::SpaceAround,
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.0, 0.0, 0.0)),
        ..Default::default()
    }
}

/// Header of the menu
fn header(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Ruspect".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/orangeJuice2.0.ttf"),
                    font_size: 80.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Logo of the game
fn logo(asset_server: &Res<AssetServer>) -> ImageBundle {
    ImageBundle {
        image: UiImage(asset_server.load("logo.png")),
        style: Style {
            size: Size::new(Val::Px(75.0), Val::Px(75.0)),
            margin: Rect {
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Menu-Options
fn menu_options_text(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Play".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/orangeJuice2.0.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                },
                TextSection {
                    value: "\nQuit".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/orangeJuice2.0.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Text after the game (score and boss slain)
fn menu_text_after_game_end(
    asset_server: &Res<AssetServer>,
    score: i32,
    boss_slain: bool,
) -> TextBundle {
    let score = &score.to_string();

    let win_or_lose = if boss_slain {
        "You won!".to_string()
    } else {
        "Game over!".to_string()
    };
    let text = format!("{} \n Your score: {}", win_or_lose, score);

    TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: text,
                style: TextStyle {
                    font: asset_server.load("fonts/orangeJuice2.0.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Verticel line for spacing
fn spacer() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.0), Val::Px(5.0)),
            position_type: PositionType::Relative,
            align_self: AlignSelf::Center,
            margin: Rect {
                top: Val::Px(40.0),
                bottom: Val::Px(40.0),
                ..Default::default()
            },
            ..Default::default()
        },
        color: UiColor(Color::rgb(0.8, 0.8, 0.8)),
        ..Default::default()
    }
}
