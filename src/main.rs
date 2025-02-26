mod camera;
mod game;
mod hierarchy;
mod state;
mod ui;

use crate::camera::CameraPlugin;
use crate::game::game::GamePlugin;
use crate::state::StatePlugin;
use crate::ui::title::home::UITitleMenuHomePlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::{default, App, AssetPlugin, ImagePlugin, PluginGroup, Window, WindowPlugin};
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};

/// SAFETY: The runtime environment must be single-threaded WASM.
// #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
// #[global_allocator]
// static ALLOCATOR: talc::TalckWasm = unsafe { talc::TalckWasm::new_global() };

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: String::from("game-assets"),
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings { ..default() }),
                ..default()
            }),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(EguiPlugin)
    .add_plugins(UITitleMenuHomePlugin)
    .add_plugins(StatePlugin)
    .add_plugins(CameraPlugin)
    .add_plugins(GamePlugin);
    if cfg!(debug_assertions) {
        app.add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());
    }
    app.run();
}
