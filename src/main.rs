use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_skein::SkeinPlugin;

fn main() {
    App::new()
        .register_type::<Character>()
        .register_type::<AutoCollider>()
        .add_plugins((
            DefaultPlugins,
            SkeinPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
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
        )
        .add_systems(Startup, startup)
        .run();
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Character {
    name: String,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct AutoCollider;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("cars/car1.gltf")),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
