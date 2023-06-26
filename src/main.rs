use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use std::ops::Mul;

const PLAYER_SPEED: f32 = 500.0;
const GRAVITATIONAL: f32 = 0.01;
const PI: f32 = 3.14;
const RADIUS_COEFFICIENT: f32 = 1000.0; // this is very nonlinear, bigger number = smaller balls
const SPAWN_VELOCITY_COEFFICIENT: f32 = 5.0;
const DEFAULT_PLANET_MASS: f32 = 10000.0;

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component, Default)]
struct Mass(f32);

#[derive(Component, Default)]
struct Radius(f32);

#[derive(Component, Default)]
struct Spawnpoint(Vec3);

#[derive(Bundle, Default)]
struct BodyBundle {
    mass: Mass,
    velocity: Velocity,
    radius: Radius,
}

#[derive(Component)]
struct IFrame(Timer);

#[derive(Component)]
struct ClickDragLine;

fn border_enforcement(
    mut body_query: Query<(&mut Transform, &mut Velocity), Without<PrimaryWindow>>,
    mut window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let x_max = window.width();
    let x_min = 0.;
    let y_max = window.height();
    let y_min = 0.;
    body_query.par_iter_mut().for_each_mut(|(mut t, mut v)| {
        // Clamp position
        t.translation.x = t.translation.x.clamp(x_min + 1., x_max - 1.);
        t.translation.y = t.translation.y.clamp(y_min + 1., y_max - 1.);

        // Force inwards
        v.0.x += -(t.translation.x - (x_max + x_min) / 2.).signum() * 1000.0
            / ((t.translation.x - x_min)
                .abs()
                .min((t.translation.x - x_max).abs())
                .powi(3));
        v.0.y += -(t.translation.y - (y_max + y_min) / 2.).signum() * 1000.0
            / (t.translation.y - y_min)
                .abs()
                .min((t.translation.y - y_max).abs())
                .powi(3);
    });
}

fn collide(
    mut commands: Commands,
    mut body_query: Query<(Entity, &Transform, &mut Velocity, &mut Mass, &Radius)>,
) {
    // dbg!(&body_query);
    let mut iter = body_query.iter_combinations_mut::<2>();

    while let Some([(e1, t1, mut v1, mut m1, r1), (e2, t2, v2, m2, r2)]) = iter.fetch_next() {
        if (t1.translation - t2.translation).length() > (r1.0 + r2.0) {
            continue;
        }
        // These two points are colliding

        // The energy threshold under which the objects simply merge together
        // This is per unit of mass (e.g. 1 unit of mass absorbs 10 KE)
        const ke_threshold: f32 = 10.0;

        // get relative velocity
        let vr = v1.0 - v2.0;
        if v1.0 == v2.0 {
            panic!("Ruh Roh!");
        }

        // Kinetic energy of each object (with respect to the other)
        let ke1 = 0.5 * vr.length_squared() * m2.0; // energy with which e2 hit e1
        let ke2 = 0.5 * vr.length_squared() * m1.0; // energy with which e1 hit e2

        // System momentum
        let mom = v1.0 * m1.0 + v2.0 * m2.0;

        // Dot product of normalized velocities
        let dot = v1.0.normalize().dot(v2.0.normalize());

        // Absorption thresholds for each mass
        let ab1 = ke_threshold * m1.0;
        let ab2 = ke_threshold * m2.0;

        if ke1 > ab1 {
            // e2 has hit e1 with more than its absorbable amount of energy
        }

        // Kinetic Energy

        // Two metrics for determining spread, splits and merging
        // Impact Force
        // Dot Product
        // These are then balanced between the two using relative mass

        // Rules: Momentum is preserved
        // We calculate the kinetic energy of the impact and deduct that from the momentum
        // Some amount of momentum is converted to energy
        // Objects have a threshold after which the more kinetic energy, the more they shatter

        // Dot Product of v1 and v2 normalized,
        // Highly positive:

        // determine the amount of divisions for each asteroid seperately
        // calculate a spread for each asteroid based on dot product
        //    Positive (1)  = minimum spread but still a little
        //    Zero (0)      = hit at a right angle, 45deg spread
        //    Negative (-1) = hit head on, 90 deg spread
    }
}

fn maintain_radius(mut body_query: Query<(&Mass, &mut Radius, &mut Transform)>) {
    for (mass, mut radius, mut transform) in body_query.iter_mut() {
        radius.0 = (mass.0 / (PI * RADIUS_COEFFICIENT)).sqrt();
        (transform.scale.x, transform.scale.y) = (radius.0, radius.0);
    }
}

fn gravity(mut body_query: Query<(&Transform, &mut Velocity, &Mass)>, time: Res<Time>) {
    let mut iter = body_query.iter_combinations_mut::<2>();

    while let Some([(t1, mut v1, m1), (t2, mut v2, m2)]) = iter.fetch_next() {
        let diff = t1.translation - t2.translation;
        let dist_sq = diff.length_squared();

        let f = GRAVITATIONAL * time.delta_seconds() / dist_sq;
        let a1 = -diff.normalize() * f * m2.0;
        let a2 = diff.normalize() * f * m1.0;

        v1.0 += a1;
        v2.0 += a2;
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn spawn_asteroid(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    spawn_query: Query<(Entity, &Spawnpoint), (With<Spawnpoint>, Without<PrimaryWindow>)>,
    buttons: Res<Input<MouseButton>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    if buttons.just_pressed(MouseButton::Right) && spawn_query.is_empty() {
        commands.spawn((Spawnpoint(cursor_pos.extend(0.0)),));
    } else if buttons.just_released(MouseButton::Right) {
        let Ok((entity, spawnpoint)) = spawn_query.get_single() else {
            return;
        };
        commands.entity(entity).remove::<Spawnpoint>();
        commands.entity(entity).insert((
            BodyBundle {
                velocity: Velocity(if spawnpoint.0 != cursor_pos.extend(0.0) {
                    (spawnpoint.0 - cursor_pos.extend(0.0)).normalize()
                        * (((spawnpoint.0 - cursor_pos.extend(0.0)).length() + 0.0) / 0.7)
                            .log(2.2)
                            .max(0.0)
                        * SPAWN_VELOCITY_COEFFICIENT
                } else {
                    Vec3::ZERO
                }),
                mass: Mass(DEFAULT_PLANET_MASS),
                ..default()
            },
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                transform: Transform {
                    translation: spawnpoint.0,
                    ..default()
                },
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                ..default()
            },
        ));
    }
}

fn maintain_spawnline(
    mut commands: Commands,
    mut line_query: Query<(&mut Visibility, &mut Path), With<ClickDragLine>>,
    window_query: Query<&Window, (With<PrimaryWindow>, Without<ClickDragLine>)>,
    spawn_query: Query<&Spawnpoint, (With<Spawnpoint>, Without<PrimaryWindow>)>,
) {
    let (Ok(window), Ok((mut visibility, mut path))) = (window_query.get_single(), line_query.get_single_mut()) else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(spawnpoint) = spawn_query.get_single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;
    let line = shapes::Line(spawnpoint.0.xy(), cursor_pos);
    *path = ShapePath::build_as(&line);
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    // Spawnline:
    let shape = shapes::Line(Vec2::ZERO, Vec2::ZERO);
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Stroke::new(Color::BLACK, 10.0),
        Fill::color(Color::RED),
        ClickDragLine,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_camera)
        .add_system(spawn_asteroid)
        .add_system(maintain_spawnline)
        .add_system(apply_velocity)
        .add_system(gravity)
        .add_system(collide)
        .add_system(maintain_radius)
        .add_system(border_enforcement)
        .run();
}
