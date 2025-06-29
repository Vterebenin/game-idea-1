use avian3d::prelude::*;
use bevy::color::palettes::tailwind::{PINK_50, RED_500};
use bevy::prelude::*;

use bevy::color::palettes::css::{BLUE, PINK, PURPLE, RED};

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
        &mut CharacterMesh,
        Entity,
    )>,
    tire_q: Query<(&Transform, &Tire, Entity)>,
    objects_q: Query<&Transform, (Without<CharacterMesh>, Without<Tire>)>,
    physics: SpatialQuery,
    mut gizmos: Gizmos,
) {
    for (transform, mut force, velocity, player, player_id) in query.iter_mut() {
        for (tire_transform, tire, _entity) in tire_q.iter() {
            let origin = transform.translation;
            let down_direction = Dir3::NEG_Y;
            let height_buffer = 0.1;
            let max_distance = player.ride_height + height_buffer;
            let query_filter =
                SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);

            debug_draw_gizmos(&mut gizmos, origin, down_direction, max_distance);

            if let Some(hit) =
                physics.cast_ray(origin, down_direction, max_distance, true, &query_filter)
            {
                let (spring_force, penetration) =
                    compute_spring_force(&player, &velocity, hit.distance, height_buffer);

                handle_grounded_state(&mut commands, player_id, penetration, height_buffer);

                let total_force = down_direction * spring_force;
                let mut force_velocity = *velocity;
                force_velocity.y = (force_velocity.y.abs() * 4.).max(4.);
                let location = tire.relative_position;
                force.apply_force_at_point(total_force, location, Vec3::new(0., 0., 0.));
                gizmos.line(
                    transform.translation + location,
                    transform.translation + location + down_direction * max_distance,
                    RED_500,
                );
                apply_impulse_to_object(
                    &mut commands,
                    &objects_q,
                    hit.entity,
                    origin,
                    down_direction,
                    *force_velocity,
                    &mut gizmos,
                );
                let slope_angle = hit.normal.angle_between(Vec3::Y).to_degrees();
                let max_slope_angle = 30.0; // Threshold for sliding
                if slope_angle > max_slope_angle {
                    let gravity = Vec3::NEG_Y;
                    let normal = hit.normal;
                    let sliding_direction = (gravity - normal * gravity.dot(normal)).normalize();
                    let sliding_force = sliding_direction * (slope_angle - max_slope_angle) * 1.5;
                    force.apply_force(sliding_force);
                }
            } else {
                commands.entity(player_id).remove::<Grounded>();
            }
        }
    }
}

fn compute_spring_force(
    player: &CharacterMesh,
    velocity: &LinearVelocity,
    hit_distance: f32,
    height_buffer: f32,
) -> (f32, f32) {
    let penetration =
        (player.ride_height + height_buffer - hit_distance).clamp(0.0, player.ride_height);

    let offset = hit_distance - player.ride_height;
    let relative_velocity = velocity.dot(*Dir3::NEG_Y);

    let spring_force = (offset * player.ride_strength) - (relative_velocity * player.ride_damper);
    // println!(
    //     "off: {} str: {} vel: {} damper: {} result: {}",
    //     offset, player.ride_strength, relative_velocity, player.ride_damper, spring_force
    // );

    (spring_force, penetration)
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
