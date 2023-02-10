use bevy::prelude::*;
use rand::Rng;

const WIDTH: f32 = 1280.0 / 3.0;
const HEIGHT: f32 = 720.0 / 3.0;

const PIPE_HEIGHT: f32 = 160.0;
const PIPE_WIDTH: f32 = 26.0;
const BIRD_HEIGHT: f32 = 12.0;
const BIRD_WIDTH: f32 = 17.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum AppState {
    Game,
    GameOver,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.8, 0.9)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: WIDTH,
                        height: HEIGHT,
                        title: "Flappy Bevy".to_string(),
                        scale_factor_override: Some(3.0),
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_state(AppState::Game)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_enter(AppState::Game).with_system(game_setup))
        .add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(jump)
                .with_system(spawn_pipes)
                .with_system(check_collisions)
                .with_system(apply_gravity)
                .with_system(apply_velocity)
                .with_system(remove_offscreen_pipes),
        )
        .add_system_set(SystemSet::on_exit(AppState::Game).with_system(scene_change_clean))
        .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(create_gameover_ui))
        .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(restart_game))
        .add_system_set(SystemSet::on_exit(AppState::GameOver).with_system(scene_change_clean))
        .run();
}

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct Pipe;

#[derive(Component, Deref, DerefMut)]
struct PipeTimer(Timer);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct CleanOnSceneChange;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Gravity(bool);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn game_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // bird
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bird.png"),
            transform: Transform {
                translation: Vec3::new(-(WIDTH / 4.0), 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        Bird,
        CleanOnSceneChange,
        Velocity(Vec2::new(0.0, 0.0)),
        Gravity(false),
    ));

    // pipe timer
    commands.spawn((
        PipeTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        CleanOnSceneChange,
    ));
}

fn spawn_pipes(
    time: Res<Time>,
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut query: Query<&mut PipeTimer>,
) {
    for mut timer in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            spawn_pipe_couple(&mut commands, &mut asset_server);
        }
    }
}

fn spawn_pipe_couple(commands: &mut Commands, asset_server: &mut Res<AssetServer>) {
    const PIPE_SPEED: f32 = 2.0;

    const MAX_HOLE_SIZE: f32 = 100.0;
    const MIN_HOLE_SIZE: f32 = 40.0;
    const MAX_HOLE_HEIGHT: f32 = HEIGHT / 4.0;
    const MIN_HOLE_HEIGHT: f32 = -HEIGHT / 4.0;

    let mut rng = rand::thread_rng();
    let hole_size = rng.gen_range(MIN_HOLE_SIZE..MAX_HOLE_SIZE);
    let hole_height = rng.gen_range(MIN_HOLE_HEIGHT..MAX_HOLE_HEIGHT);
    let top = PIPE_HEIGHT / 2.0 + hole_height + hole_size / 2.0;
    let bottom = -PIPE_HEIGHT / 2.0 + hole_height - hole_size / 2.0;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("pipe_top.png"),
            transform: Transform {
                translation: Vec3::new(WIDTH / 2.0, top, 0.0),
                ..default()
            },
            ..default()
        },
        Pipe,
        Collider,
        CleanOnSceneChange,
        Velocity(Vec2::new(-PIPE_SPEED, 0.0)),
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("pipe_bottom.png"),
            transform: Transform {
                translation: Vec3::new(WIDTH / 2.0, bottom, 0.0),
                ..default()
            },
            ..default()
        },
        Pipe,
        Collider,
        CleanOnSceneChange,
        Velocity(Vec2::new(-PIPE_SPEED, 0.0)),
    ));
}

fn check_collisions(
    mut app_state: ResMut<State<AppState>>,
    collider_query: Query<&Transform, With<Collider>>,
    bird_query: Query<(&Transform, &Bird)>,
) {
    for (bird_transform, _) in bird_query.iter() {
        for collider_transform in collider_query.iter() {
            if (collider_transform.translation.x + PIPE_WIDTH / 2.0
                > bird_transform.translation.x - BIRD_WIDTH / 2.0
                && collider_transform.translation.x - PIPE_WIDTH / 2.0
                    < bird_transform.translation.x + BIRD_WIDTH / 2.0
                && collider_transform.translation.y + PIPE_HEIGHT / 2.0
                    > bird_transform.translation.y - BIRD_HEIGHT / 2.0
                && collider_transform.translation.y - PIPE_HEIGHT / 2.0
                    < bird_transform.translation.y + BIRD_HEIGHT / 2.0)
                || bird_transform.translation.y > HEIGHT / 2.0
                || bird_transform.translation.y < -HEIGHT / 2.0
            {
                app_state.set(AppState::GameOver).ok();
            }
        }
    }
}

fn remove_offscreen_pipes(mut commands: Commands, query: Query<(Entity, &Transform), With<Pipe>>) {
    for (entity, transform) in query.iter() {
        if transform.translation.x < -WIDTH / 1.5 {
            commands.entity(entity).despawn();
        }
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Velocity, &Gravity, &Bird)>) {
    const GRAVITY: f32 = 7.0;

    for (mut velocity, gravity, _) in query.iter_mut() {
        if gravity.0 {
            velocity.0.y -= GRAVITY * time.delta_seconds();
        }
    }
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation +=
            Vec3::new(velocity.0.x, velocity.0.y, 0.0) * time.delta_seconds() * 100.0;
    }
}

fn jump(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Gravity, &Bird)>,
) {
    const JUMP_VELOCITY: f32 = 2.0;

    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut velocity, mut gravity, _) in query.iter_mut() {
            gravity.0 = true;
            velocity.0.y = JUMP_VELOCITY;
        }
    }
}

fn scene_change_clean(mut commands: Commands, query: Query<Entity, With<CleanOnSceneChange>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn create_gameover_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("game_over.png"),
            transform: Transform {
                translation: Vec3::new(0.0, 15.0, 0.0),
                ..default()
            },
            ..default()
        },
        CleanOnSceneChange,
    ));

    // restart button
    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(40.0), Val::Px(14.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px((15.0 - 7.0) + HEIGHT / 2.0),
                    left: Val::Px(WIDTH / 2.0 - 20.0),
                    ..default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: asset_server.load("gameover_ok.png").into(),
            ..default()
        },
        CleanOnSceneChange,
    ));
}

fn restart_game(
    mut app_state: ResMut<State<AppState>>,
    query: Query<&Interaction, Changed<Interaction>>,
) {
    for interaction in query.iter() {
        if let Interaction::Clicked = interaction {
            println!("Restarting game...");
            app_state.set(AppState::Game).unwrap();
        }
    }
}
