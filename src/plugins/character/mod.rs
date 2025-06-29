use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterObject>()
            .register_type::<CharacterMesh>()
            .register_type::<SceneItem>()
            .register_type::<Tire>()
            .register_type::<CharacterSpawner>()
            .add_observer(on_add_character)
            .add_systems(Update, (move_me_baby, place_tires))
            .add_systems(Update, log_transform_of_scene_items);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct SceneItem;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct CharacterSpawner;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct CharacterObject;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component, Default)]
pub struct Tire {
    relative_position: Vec3
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
pub struct CharacterMesh {
    pub ride_height: f32,
    pub ride_strength: f32,
    pub ride_damper: f32,
}

impl Default for CharacterMesh {
    fn default() -> Self {
        Self {
            ride_height: 1.6,
            ride_strength: 100.,
            ride_damper: 20.,
        }
    }
}

fn move_me_baby(
    mut player_query: Query<(&mut LinearVelocity, Entity), With<CharacterMesh>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if player_query.is_empty() {
        return;
    }

    let (mut velocity, _) = player_query.single_mut().unwrap();

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0.x += 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity.0.z -= 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity.0.x -= 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity.0.z += 0.1
    }
}

fn log_transform_of_scene_items(
    mut commands: Commands,
    scene_items_q: Query<Entity, With<SceneItem>>,
) {
    for scene_item in scene_items_q.iter() {
        // commands.get_entity(scene_item).unwrap().log_components();
    }
}

fn place_tires(
    mut tires_q: Query<(&mut Transform, &Tire), (With<Tire>, Without<CharacterObject>)>,
    character_obj_q: Single<&Transform, (Without<Tire>, With<CharacterObject>)>,
) {
    for (mut tire_transform, tire) in tires_q.iter_mut() {
        tire_transform.translation = character_obj_q.translation + tire.relative_position;
        println!("{}", tire_transform.translation)
    }
}

fn on_add_character(
    _trigger: Trigger<OnAdd, CharacterMesh>,
    spawner_transform: Single<&Transform, (With<CharacterSpawner>, Without<CharacterMesh>)>,
    mut character_transform: Single<&mut Transform, (With<CharacterMesh>, Without<CharacterSpawner>)>,
) {
    character_transform.translation = spawner_transform.translation
}
