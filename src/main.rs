use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use plugins::{camera::CameraPlugin, character::CharacterPlugin};
mod plugins;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SkeinPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_plugins((CameraPlugin, CharacterPlugin))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("cars/car1.gltf")),
    ));
}
