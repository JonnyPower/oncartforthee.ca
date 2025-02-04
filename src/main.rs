use bevy::DefaultPlugins;
use bevy::prelude::{App, default, PluginGroup, Window, WindowPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .run();
}
