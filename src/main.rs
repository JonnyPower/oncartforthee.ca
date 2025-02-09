mod state;
mod ui;

use bevy::DefaultPlugins;
use bevy::prelude::{App, default, PluginGroup, Window, WindowPlugin};
use bevy_egui::EguiPlugin;
use crate::state::StatePlugin;
use crate::ui::title::home::UITitleMenuHomePlugin;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(UITitleMenuHomePlugin)
        .add_plugins(StatePlugin)
        .run();
}
