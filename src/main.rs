use bevy::{
    prelude::*,
    sprite::{collide_aabb::collide, MaterialMesh2dBundle},
    window::WindowResolution,
};

const WIDTH: f32 = 800.;
const HEIGHT: f32 = 100.;
// const FOOD_AMOUNT: f32 = ;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Pacm1n"),
                        resolution: WindowResolution::new(WIDTH * 1.1, HEIGHT * 2.5),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_resource::<Score>()
        .add_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(
            Update,
            (
                handle_movement,
                handle_direction_change,
                eat,
                update_score,
                animate_sprite,
                handle_ghost_movement,
                catch,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), teardown)
        .add_systems(OnExit(GameState::GameOver), teardown)
        .add_systems(OnEnter(GameState::GameOver), game_over_text)
        .add_systems(Update, retry.run_if(in_state(GameState::GameOver)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut score: ResMut<Score>,
) {
    score.0 = 0;
    // pacman
    let texture_handle = asset_server.load("gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite {
                index: animation_indices.first,
                // custom_size: Some(Vec2::splat(HEIGHT * 0.75)),
                ..default()
            },
            // ::new(animation_indices.first),
            // transform: Transform::from_xyz(-WIDTH / 2., 0., 10.).with_scale(Vec3::splat(4.)),
            transform: Transform::from_scale(Vec3::splat(4.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Direction::Right,
        Pacman,
    ));

    // ghost
    let ghost_texture_handle = asset_server.load("mani-idle-run.png");
    let ghost_texture_atlas = TextureAtlas::from_grid(
        ghost_texture_handle,
        Vec2::new(24.0, 24.0),
        7,
        1,
        None,
        None,
    );
    let ghost_texture_atlas_handle = texture_atlases.add(ghost_texture_atlas);
    let ghost_animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: ghost_texture_atlas_handle,
            sprite: TextureAtlasSprite {
                index: ghost_animation_indices.first,
                // custom_size: Some(Vec2::splat(HEIGHT * 0.75)),
                ..default()
            },
            transform: Transform::from_xyz(-WIDTH / 2., 0., 10.).with_scale(Vec3::splat(4.)),
            ..default()
        },
        ghost_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Direction::Right,
        Ghost,
    ));

    // boundary
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1., 0., 0.),
            custom_size: Some(Vec2::new(WIDTH, 10.)),
            ..default()
        },
        transform: Transform::from_xyz(0., HEIGHT / 2. + 10., 0.),
        ..default()
    });
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1., 0., 0.),
            custom_size: Some(Vec2::new(WIDTH, 10.)),
            ..default()
        },
        transform: Transform::from_xyz(0., -HEIGHT / 2. - 10., 0.),
        ..default()
    });

    // scoreboard
    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        },))
        .with_children(|node| {
            node.spawn((
                ScoreText,
                TextBundle::from_section(
                    "0",
                    TextStyle {
                        // font: asset_server.load("fonts/flappybird.ttf"),
                        font_size: 80.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_text_alignment(TextAlignment::Center),
            ));
        });

    // food
    for i in ((HEIGHT as usize / 2)..(WIDTH as usize) / 2).step_by(HEIGHT as usize + 1) {
        for j in [-1., 1.] {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(HEIGHT / 8.).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::YELLOW)),
                    transform: Transform::from_translation(Vec3::new(i as f32 * j, 0., 0.)),
                    ..default()
                },
                Food,
            ));
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Default, Clone, States)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

#[derive(Component)]
struct Pacman;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct Ghost;

#[derive(Resource, Default)]
struct Score(usize);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn handle_movement(
    time: Res<Time>,
    mut sprite_position: Query<(&mut Direction, &mut Transform), With<Pacman>>,
) {
    for (direction, mut transform) in &mut sprite_position {
        match *direction {
            Direction::Left => transform.translation.x -= 150. * time.delta_seconds(),
            Direction::Right => transform.translation.x += 150. * time.delta_seconds(),
        }

        if transform.translation.x > WIDTH / 2. {
            transform.translation.x -= WIDTH;
        } else if transform.translation.x < -WIDTH / 2. {
            transform.translation.x += WIDTH;
        }
    }
}

fn handle_ghost_movement(
    time: Res<Time>,
    pacman_query: Query<&Transform, (With<Pacman>, Without<Ghost>)>,
    mut ghost_query: Query<(&mut Direction, &mut Transform, &mut TextureAtlasSprite), With<Ghost>>,
) {
    let (mut direction, mut transform, mut sprite) = ghost_query.single_mut();

    let pacman = pacman_query.single();
    if pacman.translation.x < transform.translation.x {
        *direction = Direction::Left;
        sprite.flip_x = true;
    } else {
        *direction = Direction::Right;
        sprite.flip_x = false;
    }

    // ghost can't go through boundaries
    match *direction {
        Direction::Left => {
            if transform.translation.x > -WIDTH / 2. {
                transform.translation.x -= 160. * time.delta_seconds()
            }
        }
        Direction::Right => {
            if transform.translation.x < WIDTH / 2. {
                transform.translation.x += 160. * time.delta_seconds()
            }
        }
    }
}

fn handle_direction_change(
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite_direction: Query<(&mut Direction, &mut TextureAtlasSprite), With<Pacman>>,
) {
    let (mut direction, mut sprite) = sprite_direction.single_mut();
    if keyboard_input.just_pressed(KeyCode::Space) {
        match *direction {
            Direction::Left => {
                *direction = Direction::Right;
                sprite.flip_x = false;
            }
            Direction::Right => {
                *direction = Direction::Left;
                sprite.flip_x = true;
            }
        }
    }
}

fn eat(
    pacman: Query<&Transform, With<Pacman>>,
    mut foods: Query<(&Transform, &mut Visibility), With<Food>>,
    mut score: ResMut<Score>,
) {
    let pacman = pacman.single();

    // keep count of how many food is still visible
    let mut n_vis = 0;
    for (tfm, mut vis) in &mut foods {
        if *vis == Visibility::Hidden {
            continue;
        }
        n_vis += 1;
        let collision = collide(
            pacman.translation,
            Vec2::splat(72.),
            tfm.translation,
            Vec2::splat(HEIGHT / 4.),
        );
        if collision.is_some() {
            *vis = Visibility::Hidden;
            score.0 += 1;
        }
    }

    // if all food is hidden, make them all visible
    if n_vis == 0 {
        for (_tfm, mut vis) in &mut foods {
            *vis = Visibility::Visible;
        }
    }
}

fn catch(
    pacman: Query<&Transform, With<Pacman>>,
    ghost: Query<&Transform, With<Ghost>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pacman = pacman.single();
    let ghost = ghost.single();
    let collision = collide(
        pacman.translation,
        Vec2::splat(70.), // not sure how to get the right width here
        ghost.translation,
        Vec2::splat(70.),
    );
    if collision.is_some() {
        next_state.set(GameState::GameOver);
    }
}

fn update_score(mut query: Query<&mut Text, With<ScoreText>>, score: Res<Score>) {
    if score.is_changed() {
        let mut score_text = query.single_mut();
        score_text.sections[0].value = score.0.to_string();
    }
}

fn teardown(mut commands: Commands, entities: Query<Entity, (Without<Camera>, Without<Window>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn retry(keyboard_input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

fn game_over_text(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|node| {
            node.spawn(
                TextBundle::from_section(
                    format!("Score: {}", score.0),
                    TextStyle {
                        // font: asset_server.load("fonts/flappybird.ttf"),
                        font_size: 80.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_text_alignment(TextAlignment::Center),
            );
        });
}
