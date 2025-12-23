use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use std::f32::consts::TAU;
use avian3d::prelude::*;
use rand::Rng;
use rand::rng;
use rand::prelude::IndexedRandom;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Score(0))
        .insert_resource(EnemyCount(0))
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, camera_movement, spawn_enemies, enemy_movement, hit_wall, overlay))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Car;

fn setup(
    mut commands: Commands,
    assets: ResMut<AssetServer>
) {
    // Ground
    let map_scene = assets.load("models/map/map.glb#Scene0");

    commands.spawn((
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        SceneRoot(map_scene.clone()),
        Transform::from_xyz(0.0, -1.0, 0.0)
    ));

    // Player
    let car1_scene = assets.load("models/cars/car1.glb#Scene0");
    let player_pos = Vec3::new(4.0, 2.0, 0.0);

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(2.0, 2.0, 4.5),
        Mass(1.0),
        SceneRoot(car1_scene.clone()),
        Transform::from_xyz(player_pos.x, player_pos.y, player_pos.z)
            .with_scale(Vec3::new(1.0, 1.0, 1.0)),
        MaxLinearSpeed(30.0),
        MaxAngularSpeed(8.0),
        LinearDamping(2.0),
        AngularDamping(0.5),
        LockedAxes::new()
            .lock_rotation_x()
            .lock_rotation_z(),
        Player,
        Car
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(player_pos.x, player_pos.y + 3.0, player_pos.z - 16.0)
            .with_rotation(Quat::from_rotation_x(60_f32.to_radians()))
            .with_rotation(Quat::from_rotation_y(180_f32.to_radians())),
        PlayerCamera
    ));

    // Music
    commands.spawn((
        AudioPlayer::new(assets.load("audio/bjallerklang_av_jack.ogg")),
        PlaybackSettings::LOOP
    ));

    // UI
    commands.spawn((
        Text::default(),
        TextFont {
            font_size: 25.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.9, 0.1, 0.8).into()),
    ));
}

fn camera_movement(
    mut camera: Single<&mut Transform, With<PlayerCamera>>,
    player: Single<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    camera.translation = player.translation + player.forward() * 12.0;
    camera.rotation = Quat::from_rotation_arc(Vec3::Z, player.forward().as_vec3());
    camera.translation.y += 4.0;
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(Forces, &mut Transform), With<Player>>,
    timer: Res<Time>
) {
    let speed_mov = 30.0;
    let speed_rot = 0.3;

    for (mut forces, mut transform) in player.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.rotate_y(speed_rot * TAU * timer.delta_secs());
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.rotate_y(-speed_rot * TAU * timer.delta_secs());
        }

        let mut forward = transform.forward().as_vec3();
        forward.y = 0.0;
        forces.apply_force(forward * -speed_mov);
    }
}

#[derive(Resource)]
struct EnemyCount(usize);

#[derive(Resource)]
struct Score(usize);

fn spawn_enemies(
    mut commands: Commands,
    assets: ResMut<AssetServer>,
    mut enemy_count: ResMut<EnemyCount>
) {
    let spawn_1 = Vec3::new(15.0, 2.0, 15.0);
    let spawn_2 = Vec3::new(15.0, 2.0, -15.0);
    let spawn_3 = Vec3::new(-15.0, 2.0, 15.0);
    let spawn_4 = Vec3::new(-15.0, 2.0, -15.0);
    let spawns = vec![spawn_1, spawn_2, spawn_3, spawn_4];

    let car2 = assets.load("models/cars/car2.glb#Scene0");
    let car3 = assets.load("models/cars/car3.glb#Scene0");
    let car4 = assets.load("models/cars/car4.glb#Scene0");
    let car5 = assets.load("models/cars/car5.glb#Scene0");
    let cars = vec![car2, car3, car4, car5];

    if enemy_count.0 < 5 {
        let mut rng = rng();
        let spawn = spawns.choose(&mut rng).unwrap();
        let car = cars.choose(&mut rng).unwrap();

        commands.spawn((
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 2.0, 4.5),
            Mass(1.0),
            SceneRoot(car.clone()),
            Transform::from_xyz(spawn.x, spawn.y, spawn.z),
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_z(),
            CollisionEventsEnabled,
            Enemy,
            Car
        ))
        .observe(enemy_explode);

        enemy_count.0 += 1;
    }
}

fn enemy_explode(
    event: On<CollisionStart>,
    player_query: Query<&Player>,
    mut commands: Commands,
    assets: ResMut<AssetServer>,
    mut enemy_count: ResMut<EnemyCount>,
    mut score: ResMut<Score>
) {
    let enemy_car = event.collider1;
    let player_car = event.collider2;

    if player_query.contains(player_car) {
        commands.spawn(
            AudioPlayer::new(assets.load("audio/explode.ogg"))
        );

        commands.entity(enemy_car).despawn();
        enemy_count.0 -= 1;

        score.0 += 1;
    }
}

fn enemy_movement(
    mut enemy: Query<(Forces, &mut Transform), With<Enemy>>,
    timer: Res<Time>
) {
    let speed_mov = 10.0;
    let speed_rot = 0.3;

    for (mut forces, mut transform) in enemy.iter_mut() {
        let mut forward = transform.forward().as_vec3();
        forward.y = 0.0;
        forces.apply_force(forward * -speed_mov);
        
        let mut rng = rng();
        let direction = rng.random_range(1..=2);
        match direction {
            1_i32 => transform.rotate_y(speed_rot * TAU * timer.delta_secs()),
            2_i32 => transform.rotate_y(-speed_rot * TAU * timer.delta_secs()),
            _ => todo!(),
        }
    }
}

fn hit_wall(
    mut ray_cast: MeshRayCast,
    mut car: Query<&mut Transform, With<Car>>
) {
    for mut transform in car.iter_mut() {
        let ray = Ray3d::new(transform.translation, -transform.forward());
        let hits = ray_cast.cast_ray(ray,
            &MeshRayCastSettings {
                visibility: RayCastVisibility::Any,
                ..Default::default()
            }
        );
        for hit in hits {
            if hit.1.distance < 2.5 {
                transform.rotate_y(180_f32.to_radians());
            }
        }
    }
}

fn overlay(
    mut text: Single<&mut Text>,
    score: Res<Score>
) {
    let score = score.0;
    text.0 = format!("Poäng: {score}").to_string();
}
