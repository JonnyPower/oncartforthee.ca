mod state;
mod ui;
mod camera;

use bevy::asset::AssetMetaCheck;
use bevy::DefaultPlugins;
use bevy::prelude::{App, AssetPlugin, default, PluginGroup, Window, WindowPlugin};
use bevy_egui::EguiPlugin;
use crate::camera::CameraPlugin;
use crate::state::StatePlugin;
use crate::ui::title::home::UITitleMenuHomePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(
                AssetPlugin {
                    file_path: String::from("game-assets"),
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }
            ).set(
            WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(EguiPlugin)
        .add_plugins(UITitleMenuHomePlugin)
        .add_plugins(StatePlugin)
        .add_plugins(CameraPlugin)
        .run();
}
