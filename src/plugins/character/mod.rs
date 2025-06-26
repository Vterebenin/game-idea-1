use avian3d::prelude::LinearVelocity;
use bevy::{input::mouse::MouseWheel, prelude::*, scene::SceneInstanceReady};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Character>()
            .add_systems(Update, move_me_baby)
            .add_observer(
                // log the component from the gltf spawn
                |trigger: Trigger<SceneInstanceReady>,
                 children: Query<&Children>,
                 characters: Query<&Character>| {
                    for entity in children.iter_descendants(trigger.target()) {
                        let Ok(character) = characters.get(entity) else {
                            continue;
                        };
                        info!(?character);
                    }
                },
            );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Character {
    pub ride_height: f32,
    pub ride_strength: f32,
    pub ride_damper: f32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            ride_height: 1.6,
            ride_strength: 100.,
            ride_damper: 20.,
        }
    }
}

fn move_me_baby(
    mut player_query: Query<(&mut Transform, &mut LinearVelocity, Entity), With<Character>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if player_query.is_empty() {
        return;
    }

    let (transform, mut velocity, __) = player_query.single_mut().unwrap();
    info!("{:?}", transform.translation);

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0.x -= 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity.0.z -= 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity.0.x += 0.1
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity.0.z += 0.1
    }
}
