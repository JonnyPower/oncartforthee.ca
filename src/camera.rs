use bevy::prelude::{App, Camera, Camera2d, Commands, Component, default, Plugin, Query, Startup, Window};

#[derive(Component)]
pub struct UICamera;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(
    mut commands: Commands,
    window_query: Query<&Window>,
) {
    commands.spawn(
        (
            Camera2d,
            UICamera,
        )
    );
}