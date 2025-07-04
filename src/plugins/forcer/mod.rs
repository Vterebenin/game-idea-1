use avian3d::prelude::*;
use bevy::prelude::*;

use bevy::color::palettes::css::{BLACK, BLUE, GREEN, RED};

use super::character::{CharacterMesh, Tire};

pub struct ForcerPlugin;

impl Plugin for ForcerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            .add_event::<AccelerationEvent>()
            //.add_systems(PreUpdate, keyboard_input.run_if(in_state(GameState::InGame)))
            .add_systems(Update, (on_movement_action, on_tire_rotation))
            .add_systems(FixedUpdate, apply_multi_force);
    }
}

const RAY_CAST_MAX_OFFSET: f32 = 0.1;

// #[derive(Component, Reflect, Debug)]
// #[reflect(Component)]
// pub struct ForcerPlugin {
//     name: String,
// }

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

pub enum SteeringDir {
    Left,
    Right,
}

#[derive(Event)]
pub enum MovementAction {
    Steer(SteeringDir),
}

#[derive(Event)]
pub struct AccelerationEvent(f32);

fn on_movement_action(
    mut acceleration_event_writer: EventWriter<AccelerationEvent>,
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // keys should be handled by controls
    if keyboard_input.pressed(KeyCode::KeyW) {
        acceleration_event_writer.write(AccelerationEvent(2.));
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        acceleration_event_writer.write(AccelerationEvent(-1.));
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement_event_writer.write(MovementAction::Steer(SteeringDir::Right));
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement_event_writer.write(MovementAction::Steer(SteeringDir::Left));
    }
}

const MAX_STEER_ANGLE: f32 = 45.0_f32.to_radians();

fn on_tire_rotation(
    mut movement_reader: EventReader<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Tire), (Without<CharacterMesh>, With<Tire>)>,
    char_q: Query<&Transform, (Without<Tire>, With<CharacterMesh>)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let is_steering =
        keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD);
    for car_transform in char_q.iter() {
        for (tire_transform, _) in &mut query {
            let combined_transform = car_transform.mul_transform(*tire_transform);
            let origin = combined_transform.translation;

            gizmos.line(origin, origin + *combined_transform.local_z(), RED);
            gizmos.line(origin, origin + *combined_transform.local_y(), GREEN);
            gizmos.line(origin, origin + *combined_transform.local_x(), BLUE);
        }
    }

    for event in movement_reader.read() {
        if let MovementAction::Steer(dir) = event {
            for (mut transform, tire) in &mut query {
                if !tire.is_front {
                    continue;
                }

                let current_angle = transform.rotation.to_euler(EulerRot::YXZ).0;
                let rotation_amount = match dir {
                    SteeringDir::Left => 0.03_f32.min(MAX_STEER_ANGLE - current_angle),
                    SteeringDir::Right => -0.03_f32.min(current_angle - (-MAX_STEER_ANGLE)),
                };

                if rotation_amount.abs() > f32::EPSILON {
                    transform.rotate(Quat::from_rotation_y(rotation_amount));
                }
            }
        }
    }

    if !is_steering {
        for (mut transform, _tire) in &mut query {
            // Get current rotation and lerp back to identity (0 rotation)
            let current_rot = transform.rotation;
            let target_rot = Quat::IDENTITY;
            let return_speed = 3.0; // Adjust this value for faster/slower return
            transform.rotation = current_rot.slerp(target_rot, return_speed * time.delta_secs());
        }
    }
}

fn apply_multi_force(
    mut acceleration_reader: EventReader<AccelerationEvent>,
    mut query: Query<(
        &Transform,
        &mut ExternalForce,
        &mut LinearVelocity,
        &AngularVelocity,
        &CharacterMesh,
        &mut LinearDamping,
        &Mass,
        Entity,
    )>,
    tire_q: Query<(&Transform, &ShapeCaster, &ShapeHits), With<Tire>>,
    physics: SpatialQuery,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let acceleration_events = acceleration_reader.read().collect::<Vec<_>>();
    for (
        car_transform,
        mut force,
        car_velocity,
        car_ang_velocity,
        car_params,
        mut car_damping,
        car_mass,
        player_id,
    ) in query.iter_mut()
    {
        let mut tires_on_ground = 0;
        for (tire_transform, shape_caster, shape_hits) in tire_q.iter() {
            let combined_transform = car_transform.mul_transform(*tire_transform);
            let tire_position = combined_transform.translation;

            let query_filter = get_filter(player_id);
            let ray_hit = get_wheel_ray_hit(
                &physics,
                car_params.ride_height,
                &query_filter,
                combined_transform,
            );

            if shape_hits.is_empty() {
                continue;
            }
            tires_on_ground += 1;
            let mut distance = 0.;
            for hit in shape_hits.iter() {
                distance = hit.distance;
            }

            let mut accel_force = Vec3::new(0., 0., 0.);
            for accel_event in &acceleration_events {
                accel_force = get_accel_force(
                    combined_transform,
                    *car_transform,
                    &car_velocity,
                    accel_event.0,
                );
            }

            let steering_force = get_steering_force(
                combined_transform,
                &car_velocity,
                car_ang_velocity,
                car_mass,
            );

            let spring_force = get_spring_force(
                combined_transform,
                &car_velocity,
                car_ang_velocity,
                car_params,
                distance,
            );

            gizmos.arrow(
                tire_position,
                tire_position + distance * combined_transform.down(),
                BLACK,
            );
            gizmos.arrow(tire_position, tire_position + spring_force, GREEN);
            gizmos.arrow(tire_position, tire_position + steering_force, RED);
            gizmos.arrow(tire_position, tire_position + accel_force, BLUE);

            force.apply_force_at_point(spring_force, tire_position, car_transform.translation);
            force.apply_force_at_point(steering_force, tire_position, car_transform.translation);
            force.apply_force_at_point(accel_force, tire_position, car_transform.translation);
        }
        if tires_on_ground > 0 {
            car_damping.0 = 0.5;
        } else {
            car_damping.0 = 0.;
        }
    }
}

fn get_filter(player_id: Entity) -> SpatialQueryFilter {
    SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id])
}

fn get_wheel_ray_hit(
    physics: &SpatialQuery,
    ride_height: f32,
    query_filter: &SpatialQueryFilter,
    combined_transform: Transform,
) -> Option<RayHitData> {
    let down_direction = -combined_transform.local_y();
    let max_distance = ride_height + RAY_CAST_MAX_OFFSET;
    physics.cast_ray(
        combined_transform.translation,
        down_direction,
        max_distance,
        true,
        query_filter,
    )
}

fn get_accel_force(
    combined_transform: Transform,
    car_transform: Transform,
    car_velocity: &LinearVelocity,
    accel_input: f32,
) -> Vec3 {
    let accel_dir = combined_transform.forward();
    let car_top_speed = 55.;
    let car_speed = (car_transform.forward().dot(**car_velocity)).abs().min(0.1);
    let speed_factor = 1.0 - (car_speed / car_top_speed).powi(2);
    let available_torque = speed_factor * accel_input * 4.;
    accel_dir * available_torque
}

fn get_steering_force(
    combined_transform: Transform,
    car_velocity: &LinearVelocity,
    car_ang_velocity: &AngularVelocity,
    car_mass: &Mass,
) -> Vec3 {
    let tire_grip_factor = 0.05;

    let steering_dir = combined_transform.right();
    let tire_position = combined_transform.translation;
    let tire_vel = car_velocity.0 + car_ang_velocity.cross(tire_position);

    let steering_vel = steering_dir.dot(tire_vel);

    let desired_vel_change = tire_grip_factor * -steering_vel;

    let desired_accel = desired_vel_change;
    // 4. is the number of tires
    let tire_mass = **car_mass / 4.;
    steering_dir * tire_mass * desired_accel
}

fn get_spring_force(
    combined_transform: Transform,
    car_velocity: &LinearVelocity,
    car_ang_velocity: &AngularVelocity,
    car_mesh: &CharacterMesh,
    distance: f32,
) -> Vec3 {
    // get velocity at point of tire
    let tire_position = combined_transform.translation;
    let tire_vel = car_velocity.0 + car_ang_velocity.cross(tire_position);
    let spring_dir = combined_transform.up();

    let offset = car_mesh.ride_height - distance;
    println!("{}", offset);
    let relative_velocity = spring_dir.dot(tire_vel);

    let spring_force =
        (offset * car_mesh.ride_strength) - (relative_velocity * car_mesh.ride_damper);

    spring_dir * spring_force
}
