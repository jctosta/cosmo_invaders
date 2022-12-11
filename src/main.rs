use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;

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
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .add_startup_system(setup)
        .add_system(player)
        .add_system(bug_movement)
        .add_system(laser_movement)
        .add_system(bug_zapper)
        .run();
}

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
                    sprite: TextureAtlasSprite::new(2),
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

fn setup(
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

    // Spawn the camera
    commands.spawn(Camera2dBundle::default());

    // Spawn the player
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, -(WINDOW_HEIGHT / 2.5), 0.0)),
            sprite: TextureAtlasSprite::new(0),
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
                    sprite: TextureAtlasSprite::new(1),
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
