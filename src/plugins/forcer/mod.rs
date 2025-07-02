use avian3d::parry::na::clamp;
use avian3d::prelude::*;
use bevy::color::palettes::tailwind::{PINK_50, RED_500};
use bevy::gizmos::gizmos;
use bevy::prelude::*;

use bevy::color::palettes::css::{BLACK, BLUE, GREEN, PINK, PURPLE, RED};

use super::character::{CharacterMesh, Tire};

pub struct ForcerPlugin;

impl Plugin for ForcerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>()
            //.add_systems(PreUpdate, keyboard_input.run_if(in_state(GameState::InGame)))
            .add_systems(Update, (on_movement_action, on_tire_rotation))
            .add_systems(
                FixedUpdate,
                (
                    // movement,
                    // apply_movement_damping,
                    // update_coyote_time,
                    apply_spring_force,
                    apply_steering_force,
                    apply_acceleration_force,
                    // apply_movement_damping,
                )
                    .chain(),
            );
    }
}

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
    Accelerate(f32),
}

fn apply_spring_force(
    mut commands: Commands,
    mut query: Query<(
        &Transform,
        &mut ExternalForce,
        &mut LinearVelocity,
        &AngularVelocity,
        &mut CharacterMesh,
        Entity,
    )>,
    tire_q: Query<(&Transform, &Tire, Entity)>,
    objects_q: Query<&Transform, (Without<CharacterMesh>, Without<Tire>)>,
    physics: SpatialQuery,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    for (car_transform, mut force, velocity, ang_vel, player, player_id) in query.iter_mut() {
        for (tire_transform, tire, _entity) in tire_q.iter() {
            let combined_transform = car_transform.mul_transform(*tire_transform);
            let tire_position = combined_transform.translation;
            let down_direction = combined_transform.down();
            let max_distance = player.ride_height;
            let query_filter =
                SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);

            gizmos.arrow(
                tire_position,
                tire_position + max_distance * down_direction,
                BLACK,
            );
            if let Some(hit) = physics.cast_ray(
                tire_position,
                down_direction,
                max_distance,
                true,
                &query_filter,
            ) {
                // get velocity at point of tire
                let vel = velocity.0 + ang_vel.cross(tire_position);
                let spring_dir = combined_transform.up();

                let offset = player.ride_height - hit.distance;
                let relative_velocity = spring_dir.dot(vel);

                let spring_force =
                    (offset * player.ride_strength) - (relative_velocity * player.ride_damper);

                let total_force = spring_dir * spring_force;
                force.apply_force_at_point(total_force, tire_position, car_transform.translation);
                gizmos.arrow(tire_position, tire_position + total_force, GREEN);
            } else {
                commands.entity(player_id).remove::<Grounded>();
            }
        }
    }
}

fn apply_steering_force(
    mut query: Query<(
        &Transform,
        &mut ExternalForce,
        &mut LinearVelocity,
        &AngularVelocity,
        &mut CharacterMesh,
        &Mass,
        Entity,
    )>,
    tire_q: Query<(&Transform, &Tire, Entity)>,
    physics: SpatialQuery,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let tire_grip_factor = 0.3;

    for (car_transform, mut force, velocity, ang_vel, player, mass, player_id) in query.iter_mut() {
        for (tire_transform, _tire, _entity) in tire_q.iter() {
            let combined_transform = car_transform.mul_transform(*tire_transform);
            let tire_position = combined_transform.translation;
            let steering_dir = -combined_transform.local_z();

            let down_direction = -combined_transform.local_y();
            let max_distance = player.ride_height;
            let query_filter =
                SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);
            if physics
                .cast_ray(
                    tire_position,
                    down_direction,
                    max_distance,
                    true,
                    &query_filter,
                )
                .is_some()
            {
                let tire_vel = velocity.0 + ang_vel.cross(tire_position);

                let steering_vel = steering_dir.dot(tire_vel);

                let desired_vel_change = tire_grip_factor * -steering_vel;

                let desired_accel = desired_vel_change;
                // 4. is the number of tires
                let tire_mass = **mass / 4. / 20.;
                let total_force = steering_dir * tire_mass * desired_accel;
                force.apply_force_at_point(total_force, tire_position, car_transform.translation);
                gizmos.arrow(
                    tire_position,
                    tire_position + total_force,
                    RED,
                );
            }
        }
    }
}

fn on_movement_action(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // keys should be handled by controls
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement_event_writer.write(MovementAction::Accelerate(2.));
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement_event_writer.write(MovementAction::Accelerate(-1.));
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

fn apply_acceleration_force(
    mut movement_reader: EventReader<MovementAction>,
    mut query: Query<(
        &Transform,
        &mut ExternalForce,
        &mut LinearVelocity,
        &AngularVelocity,
        &mut CharacterMesh,
        &Mass,
        Entity,
    )>,
    tire_q: Query<(&Transform, &Tire, Entity)>,
    physics: SpatialQuery,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    for (car_transform, mut force, velocity, ang_vel, player, mass, player_id) in query.iter_mut() {
        for event in movement_reader.read() {
            if let MovementAction::Accelerate(accel_input) = event {
                for (tire_transform, _tire, _entity) in tire_q.iter() {
                    let combined_transform = car_transform.mul_transform(*tire_transform);
                    let tire_position = combined_transform.translation;

                    let down_direction = -combined_transform.local_y();
                    let max_distance = player.ride_height;
                    let query_filter =
                        SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);
                    if physics
                        .cast_ray(
                            tire_position,
                            down_direction,
                            max_distance,
                            true,
                            &query_filter,
                        )
                        .is_some()
                    {
                        let accel_dir = combined_transform.local_x();
                        let car_top_speed = 20.;
                        let car_speed = (car_transform.forward().dot(**velocity)).abs().max(0.1);
                        let normalized_speed = clamp(car_speed.abs() / car_top_speed, 0., 1.);
                        let available_torque = normalized_speed * accel_input * 40.;
                        let total_force = accel_dir * available_torque;

                        println!(
                            "{} {} {} {}",
                            total_force, car_speed, normalized_speed, available_torque
                        );
                        force.apply_force_at_point(
                            total_force,
                            tire_position,
                            car_transform.translation,
                        );
                        gizmos.arrow(tire_position, tire_position + total_force, BLUE);
                    }
                }
            }
        }
    }
}

fn apply_movement_damping(mut query: Query<&mut LinearVelocity>) {
    let factor = 0.9;
    for mut linear_velocity in &mut query {
        linear_velocity.x *= factor;
        linear_velocity.z *= factor;
    }
}
