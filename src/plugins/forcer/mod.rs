use avian3d::prelude::*;
use bevy::color::palettes::tailwind::{PINK_50, RED_500};
use bevy::prelude::*;

use bevy::color::palettes::css::{BLACK, BLUE, PINK, PURPLE, RED};

use super::character::{CharacterMesh, Tire};

pub struct ForcerPlugin;

impl Plugin for ForcerPlugin {
    fn build(&self, app: &mut App) {
        app // .add_event::<MovementAction>()
            //.add_systems(PreUpdate, keyboard_input.run_if(in_state(GameState::InGame)))
            .add_systems(
                FixedUpdate,
                (
                    // movement,
                    // apply_movement_damping,
                    // update_coyote_time,
                    apply_spring_force,
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
    for (transform, mut force, velocity, ang_vel, player, player_id) in query.iter_mut() {
        for (tire_transform, tire, _entity) in tire_q.iter() {
            let rotation = transform.rotation * tire.relative_position;
            let origin = rotation + transform.translation;
            let down_direction = transform.down();
            let max_distance = player.ride_height + 0.1;
            let query_filter =
                SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);
            debug_draw_gizmos(&mut gizmos, origin, down_direction, max_distance);

            let location = transform.rotation * tire.relative_position + transform.translation;
            gizmos.sphere(location, 0.2, BLACK);
            if let Some(hit) =
                physics.cast_ray(origin, down_direction, max_distance, true, &query_filter)
            {
                // get velocity at point of tire
                let vel = velocity.0 + ang_vel.cross(location);
                let spring_force =
                    compute_spring_force(&player, &vel, hit.distance, transform.up());

                println!("{}", spring_force);
                let total_force = transform.up() * spring_force;
                force.apply_force_at_point(total_force, location, transform.translation);
                gizmos.line(
                    location,
                    location + down_direction * max_distance,
                    RED_500,
                );
                // apply_impulse_to_object(
                //     &mut commands,
                //     &objects_q,
                //     hit.entity,
                //     origin,
                //     down_direction,
                //     *force_velocity,
                //     &mut gizmos,
                // );
                // let slope_angle = hit.normal.angle_between(Vec3::Y).to_degrees();
                // let max_slope_angle = 30.0; // Threshold for sliding
                // if slope_angle > max_slope_angle {
                //     let gravity = Vec3::NEG_Y;
                //     let normal = hit.normal;
                //     let sliding_direction = (gravity - normal * gravity.dot(normal)).normalize();
                //     let sliding_force = sliding_direction * (slope_angle - max_slope_angle) * 1.5;
                //     force.apply_force(sliding_force);
                // }
            } else {
                commands.entity(player_id).remove::<Grounded>();
            }
        }
    }
}

fn compute_spring_force(
    player: &CharacterMesh,
    velocity: &Vec3,
    hit_distance: f32,
    direction: Dir3,
) -> f32 {

    let offset = player.ride_height - hit_distance;
    let relative_velocity = direction.dot(*velocity);

    (offset * player.ride_strength) - (relative_velocity * player.ride_damper)
    // println!(
    //     "off: {} str: {} vel: {} damper: {} result: {}",
    //     offset, player.ride_strength, relative_velocity, player.ride_damper, spring_force
    // );
}

fn handle_grounded_state(
    commands: &mut Commands,
    player_id: Entity,
    penetration: f32,
    buffer: f32,
) {
    if penetration > buffer {
        commands.entity(player_id).insert(Grounded);
    } else {
        commands.entity(player_id).remove::<Grounded>();
    }
}

fn apply_impulse_to_object(
    commands: &mut Commands,
    objects_q: &Query<&Transform, (Without<CharacterMesh>, Without<Tire>)>,
    hit_entity: Entity,
    origin: Vec3,
    down_direction: Dir3,
    total_force: Vec3,
    gizmos: &mut Gizmos,
) {
    if let Ok(transform) = objects_q.get(hit_entity) {
        let mut impulse = ExternalImpulse::default();
        let point = origin + down_direction * total_force.length();
        let impulse_value = (total_force * *down_direction * 0.3)
            .clamp(Vec3::new(0., -4., 0.), Vec3::new(0., 1., 0.));

        impulse.apply_impulse_at_point(impulse_value, point, transform.translation);

        gizmos.sphere(point, 0.1, PURPLE);
        gizmos.line(point, point + impulse_value, PINK);

        commands.entity(hit_entity).insert(impulse);
    }
}

fn debug_draw_gizmos(gizmos: &mut Gizmos, origin: Vec3, direction: Dir3, max_distance: f32) {
    gizmos.line(origin, origin + direction * max_distance, BLUE);
}
