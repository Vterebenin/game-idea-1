use avian3d::prelude::*;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    scene::SceneInstanceReady,
};

use crate::plugins::character;

use super::character::CharacterObject;

const MIN_PITCH: f32 = -89.0f32.to_radians();
const MAX_PITCH: f32 = 89.0f32.to_radians();
const MOUSE_SENSITIVITY_X: f32 = 0.002;
const MOUSE_SENSITIVITY_Y: f32 = 0.002;
const ZOOM_SPEED: f32 = 10.0;
// pub const PLAYER_CAPSULE_RADIUS: f32 = 0.4;
// pub const PLAYER_CAPSULE_LENGTH: f32 = 1.2;
// pub const PLAYER_TOTAL_HEIGHT: f32 = PLAYER_CAPSULE_LENGTH + PLAYER_CAPSULE_RADIUS * 2.;

#[derive(Component)]
pub struct CameraTarget;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct UseCameraMark;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct PlayerCameraFix {
    x: f32,
    y: f32,
    z: f32,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

type WithCameraTarget = (With<CameraTarget>, Without<UseCameraMark>, Without<CharacterObject>);
type WithCamera = (With<UseCameraMark>, Without<CharacterObject>);

fn on_add_camera_mark(
    _trigger: Trigger<SceneInstanceReady>,
    camera_q: Query<Entity, WithCamera>,
    mut commands: Commands,
    character_q: Query<&Transform, With<CharacterObject>>,
) {
    let character_transform = character_q.single().unwrap();
    let camera_entity = camera_q.single().unwrap();

    let offset = Transform::from_xyz(0.0, 0.0, 0.0).translation;
    let target_translation = character_transform.translation - offset;

    commands.entity(camera_entity).insert((
        Camera3d::default(),
        PlayerCameraFix {
            x: 10.,
            y: 10.,
            z: 10.,
            distance: 10.,
            yaw: 0.,
            pitch: 0.,
        },
        Transform::from_translation(target_translation).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let offset = Transform::from_xyz(0.0, 0.0, 0.0).translation;
    let target_translation = character_transform.translation - offset;
    commands.spawn((
        CameraTarget,
        Transform::from_translation(target_translation),
    ));
}

fn rotate_camera(
    mut camera_query: Query<(&mut Transform, &mut PlayerCameraFix), WithCamera>,
    mut camera_target_q: Query<&mut Transform, WithCameraTarget>,
    mut player_query: Query<(&mut Transform, &LinearVelocity, Entity), With<CharacterObject>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    physics: SpatialQuery,
) {
    // println!("{} {}", player_query.is_empty(), camera_query.is_empty());
    if player_query.is_empty() || camera_query.is_empty() {
        return;
    }

    let (mut camera_transform, mut camera) = camera_query.single_mut().unwrap();
    let (player_transform, velocity, player_id) = player_query.single_mut().unwrap();

    for event in mouse_motion_events.read() {
        let x_delta = event.delta.x;
        let y_delta = event.delta.y;
        camera.yaw += x_delta * MOUSE_SENSITIVITY_X;
        camera.pitch += y_delta * MOUSE_SENSITIVITY_Y;
        camera.pitch = camera.pitch.clamp(MIN_PITCH, MAX_PITCH);
    }

    for event in mouse_wheel.read() {
        camera.distance -= event.y * ZOOM_SPEED * time.delta_secs();
        camera.distance = camera.distance.clamp(2.0, 20.0); // Clamp zoom levels
    }

    let offset = Vec3::new(
        camera.distance * camera.yaw.cos() * camera.pitch.cos(),
        camera.distance * camera.pitch.sin(),
        camera.distance * camera.yaw.sin() * camera.pitch.cos(),
    );

    // TODO: probably i can create some
    // sort of aiming based on this offset to the side
    // plus playing a bit with current fov
    let player_translation = player_transform.translation;
    let mut desired_position = player_translation + offset;

    let direction = desired_position - player_translation;
    let query_filter = SpatialQueryFilter::from_mask(0b1011).with_excluded_entities([player_id]);

    // shape cast if camera clipping on colliders
    if let Ok(direction) = Dir3::new(direction.normalize()) {
        if let Some(hit) = physics.cast_shape(
            &Collider::sphere(0.5),
            player_translation,
            Quat::IDENTITY,
            direction,
            &ShapeCastConfig {
                max_distance: camera.distance,
                target_distance: 0.,
                ignore_origin_penetration: true,
                ..Default::default()
            },
            &query_filter,
        ) {
            desired_position = player_translation + direction * (hit.distance - 0.1);
        }
    }

    camera_transform.translation = desired_position;
    let mut camera_target = camera_target_q.single_mut().unwrap();
    camera_target.translation = player_translation - offset;
    camera_transform.look_at(camera_target.translation, Vec3::Y);

}

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerCameraFix>()
            .register_type::<UseCameraMark>()
            .add_observer(on_add_camera_mark)
            .add_systems(Update, rotate_camera);
    }
}
