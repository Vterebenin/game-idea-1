use bevy::{prelude::*, scene::SceneInstanceReady};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Character>()
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
    name: String,
}
