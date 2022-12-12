use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Splash,
    Menu,
    Game,
}

mod splash {
    use bevy::prelude::*;

    use super::{despawn_screen, GameState};

    pub struct SplashPlugin;

    impl Plugin for SplashPlugin {
        fn build(&self, app: &mut App) {
            app.add_system_set(SystemSet::on_enter(GameState::Splash).with_system(splash_setup))
                .add_system_set(SystemSet::on_update(GameState::Splash).with_system(countdown))
                .add_system_set(
                    SystemSet::on_exit(GameState::Splash)
                        .with_system(despawn_screen::<OnSplashScreen>),
                );
        }
    }

    #[derive(Component)]
    struct OnSplashScreen;

    #[derive(Resource, Deref, DerefMut)]
    struct SplashTimer(Timer);

    fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let icon = asset_server.load("branding/logo.png");

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    ..default()
                },
                OnSplashScreen,
            ))
            .with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Auto),
                        ..default()
                    },
                    image: icon.into(),
                    ..default()
                });
            });

        commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
    }

    fn countdown(
        mut game_state: ResMut<State<GameState>>,
        time: Res<Time>,
        mut timer: ResMut<SplashTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Menu).unwrap();
        }
    }
}

mod menu {
    use bevy::{app::AppExit, prelude::*};

    use super::{despawn_screen, GameState, TEXT_COLOR};

    pub struct MenuPlugin;

    impl Plugin for MenuPlugin {
        fn build(&self, app: &mut App) {
            app.add_state(MenuState::Disabled)
                .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(menu_setup))
                .add_system_set(SystemSet::on_enter(MenuState::Main).with_system(main_menu_setup))
                .add_system_set(
                    SystemSet::on_exit(MenuState::Main)
                        .with_system(despawn_screen::<OnMainMenuScreen>),
                )
                .add_system_set(
                    SystemSet::on_enter(MenuState::Settings).with_system(settings_menu_setup),
                )
                .add_system_set(
                    SystemSet::on_exit(MenuState::Settings)
                        .with_system(despawn_screen::<OnSettingsMenuScreen>),
                )
                .add_system_set(
                    SystemSet::on_update(GameState::Menu)
                        .with_system(menu_action)
                        .with_system(button_system),
                );
        }
    }

    #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    enum MenuState {
        Main,
        Settings,
        Disabled,
    }

    #[derive(Component)]
    struct OnMainMenuScreen;

    #[derive(Component)]
    struct OnSettingsMenuScreen;

    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

    // Tag component used to mark wich setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    // All actions that can be triggered from a button click
    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Settings,
        BackToMainMenu,
        BackToSettings,
        Quit,
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, selected) in &mut interaction_query {
            *color = match (*interaction, selected) {
                (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    // fn setting_button<T: Resource + Component + PartialEq + Copy>(
    //     interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    //     mut selected_query: Query<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    //     mut commands: Commands,
    //     mut setting: ResMut<T>,
    // ) {
    //     for (interaction, button_setting, entity) in &interaction_query {
    //         if *interaction == Interaction::Clicked && *setting != *button_setting {
    //             let (previous_button, mut previous_color) = selected_query.single_mut();
    //             *previous_color = NORMAL_BUTTON.into();
    //             commands.entity(previous_button).remove::<SelectedOption>();
    //             commands.entity(entity).insert(SelectedOption);
    //             *setting = *button_setting;
    //         }
    //     }
    // }

    fn menu_setup(mut menu_state: ResMut<State<MenuState>>) {
        let _ = menu_state.set(MenuState::Main);
    }

    fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font = asset_server.load("fonts/Monocraft.otf");
        // Common style for all buttons on the screen
        let button_style = Style {
            size: Size::new(Val::Px(250.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_icon_style = Style {
            size: Size::new(Val::Px(30.0), Val::Auto),
            // This takes the icons out of the flexbox flow, to be positioned exactly
            position_type: PositionType::Absolute,
            // The icon will be close to the left border of the button
            position: UiRect {
                left: Val::Px(10.0),
                right: Val::Auto,
                top: Val::Auto,
                bottom: Val::Auto,
            },
            ..default()
        };
        let button_text_style = TextStyle {
            font: font.clone(),
            font_size: 30.0,
            color: TEXT_COLOR,
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMainMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::LIME_GREEN.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Display the game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Cosmo Invaders",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 80.0,
                                    color: TEXT_COLOR,
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );

                        // Display three buttons for each action available from the main menu:
                        // - new game
                        // - settings
                        // - quit
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/Game Icons/right.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    image: icon.into(),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "New Game",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Settings,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/Game Icons/wrench.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    image: icon.into(),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "Settings",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/Game Icons/exitRight.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style,
                                    image: icon.into(),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section("Quit", button_text_style));
                            });
                    });
            });
    }

    fn settings_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let button_style = Style {
            size: Size::new(Val::Px(200.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font: asset_server.load("fonts/Monocraft.otf"),
            font_size: 40.0,
            color: TEXT_COLOR,
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnSettingsMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::LIME_GREEN.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        for (action, text) in [(MenuButtonAction::BackToMainMenu, "Back")] {
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: button_style.clone(),
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    action,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        text,
                                        button_text_style.clone(),
                                    ));
                                });
                        }
                    });
            });
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<State<MenuState>>,
        mut game_state: ResMut<State<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Clicked {
                match menu_button_action {
                    MenuButtonAction::Quit => app_exit_events.send(AppExit),
                    MenuButtonAction::Play => {
                        game_state.set(GameState::Game).unwrap();
                        menu_state.set(MenuState::Disabled).unwrap();
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings).unwrap(),
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main).unwrap(),
                    MenuButtonAction::BackToSettings => {
                        menu_state.set(MenuState::Settings).unwrap();
                    }
                }
            }
        }
    }
}

mod game {
    use bevy::prelude::*;

    use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

    use super::{despawn_screen, GameState};

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(game_setup))
                .add_system_set(
                    SystemSet::on_update(GameState::Game)
                        .with_system(player)
                        .with_system(bug_movement)
                        .with_system(laser_movement)
                        .with_system(bug_zapper),
                )
                .add_system_set(
                    SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<OnGameScreen>),
                );
        }
    }

    #[derive(Component)]
    struct OnGameScreen;

    #[derive(Resource, Deref, DerefMut)]
    struct GameTimer(Timer);

    #[derive(Component)]
    struct Player {
        delta_x: f32,
    }

    #[derive(Copy, Clone)]
    enum BugMovement {
        Left,
        Right,
        Down { n: f32, next_left: bool },
    }

    #[derive(Component)]
    struct Bug {
        movement: BugMovement,
    }

    #[derive(Component)]
    struct Laser;

    fn player(
        keyboard_input: Res<Input<KeyCode>>,
        mut commands: Commands,
        mut query: Query<(&mut Player, &mut Transform, &Handle<TextureAtlas>)>,
    ) {
        const ACCELERATION: f32 = 1.0;
        const MAX_VELOCITY: f32 = 16.0;

        for (mut player, mut trans, atlas_handle) in query.iter_mut() {
            let mut firing = false;

            if keyboard_input.pressed(KeyCode::Left) {
                player.delta_x -= ACCELERATION;
            }
            if keyboard_input.pressed(KeyCode::Right) {
                player.delta_x += ACCELERATION;
            }
            if keyboard_input.just_pressed(KeyCode::Space) {
                firing = true;
            }

            // Apply movement deltas
            player.delta_x = player.delta_x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
            trans.translation.x += player.delta_x;
            trans.translation.x = trans
                .translation
                .x
                .clamp(-(WINDOW_WIDTH / 2.5), WINDOW_WIDTH / 2.5);

            // Decelerate
            player.delta_x *= 0.75;

            if firing {
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: atlas_handle.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            trans.translation.x,
                            trans.translation.y + 24.0,
                            0.0,
                        )),
                        sprite: TextureAtlasSprite {
                            index: 2,
                            color: Color::LIME_GREEN,
                            ..default()
                        },
                        ..default()
                    },
                    Laser,
                ));
            }
        }
    }

    fn bug_movement(mut query: Query<(&mut Bug, &mut Transform)>) {
        for (mut bug, mut trans) in query.iter_mut() {
            match bug.movement {
                BugMovement::Left => {
                    trans.translation.x -= 2.0;
                    if trans.translation.x < -(WINDOW_WIDTH / 2.5) {
                        bug.movement = BugMovement::Down {
                            n: 12.0,
                            next_left: false,
                        };
                    }
                }
                BugMovement::Right => {
                    trans.translation.x += 2.0;
                    if trans.translation.x > WINDOW_WIDTH / 2.5 {
                        bug.movement = BugMovement::Down {
                            n: 12.0,
                            next_left: true,
                        };
                    }
                }
                BugMovement::Down { n, next_left } => {
                    trans.translation.y -= 2.0;
                    bug.movement = BugMovement::Down {
                        n: n - 1.0,
                        next_left,
                    };
                    if n < 1.0 {
                        bug.movement = if next_left {
                            BugMovement::Left
                        } else {
                            BugMovement::Right
                        };
                    }
                }
            }
        }
    }

    fn laser_movement(mut query: Query<(Entity, &Laser, &mut Transform)>, mut commands: Commands) {
        for (entity, _, mut trans) in query.iter_mut() {
            trans.translation += Vec3::new(0.0, 4.0, 0.0);

            if trans.translation.y > WINDOW_HEIGHT / 2.0 {
                commands.entity(entity).despawn();
            }
        }
    }

    fn bug_zapper(
        laser_query: Query<(Entity, &Laser, &Transform)>,
        collider_query: Query<(Entity, &Bug, &Transform)>,
        mut commands: Commands,
    ) {
        for (entity, _, trans) in laser_query.iter() {
            let laser_pos = Vec2::new(trans.translation.x, trans.translation.y);
            for (bug_entity, _, bug_transform) in collider_query.iter() {
                let bug_pos = Vec2::new(bug_transform.translation.x, bug_transform.translation.y);

                if bug_pos.distance(laser_pos) < 24.0 {
                    commands.entity(bug_entity).despawn();
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    fn game_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        // Setup the sprite sheet
        let texture_handle = asset_server.load("spritesheet.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(24.0, 24.0),
            3,
            1,
            Some(Vec2::new(0.0, 0.0)),
            Some(Vec2::new(0.0, 0.0)),
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        // Spawn the player
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, -(WINDOW_HEIGHT / 2.5), 0.0)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    color: Color::LIME_GREEN,
                    ..default()
                },
                ..default()
            },
            Player { delta_x: 0.0 },
        ));

        // Spawn rows of enemies
        for bug_row in 0..4 {
            let y = 200.0 - (bug_row as f32 * 30.0);
            for bug_col in 0..20 {
                let x = -300.0 + (bug_col as f32 * 30.0);

                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle.clone(),
                        transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                        sprite: TextureAtlasSprite {
                            index: 1,
                            color: Color::LIME_GREEN,
                            ..default()
                        },
                        ..default()
                    },
                    Bug {
                        movement: if bug_row % 2 == 0 {
                            BugMovement::Left
                        } else {
                            BugMovement::Right
                        },
                    },
                ));
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                title: "Cosmo Invaders".into(),
                ..default()
            },
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_state(GameState::Splash)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn(Camera2dBundle::default());
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
