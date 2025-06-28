use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_skein::SkeinPlugin;
use plugins::{camera_controller::CameraControllerPlugin, character::CharacterPlugin, forcer::ForcerPlugin};
mod plugins;

fn main() {
    let list_of_external_plugins = (
        DefaultPlugins,
        SkeinPlugin::default(),
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new(),
    );
    let list_of_custom_plugins = (CameraControllerPlugin, CharacterPlugin, ForcerPlugin);

    App::new()
        .add_plugins(list_of_external_plugins)
        .add_plugins(list_of_custom_plugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // todo: move to scene loader
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("cars/car1.gltf")),
    ));
}
