use bevy::prelude::{App, Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Component, default, Plugin, Query, Startup, Transform, Vec3, Window};

#[derive(Component)]
pub struct UICamera;

#[derive(Component)]
pub struct GameCamera;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(
    mut commands: Commands
) {
    commands.spawn(
        (
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            GameCamera,
        )
    );
    commands.spawn(
        (
            Camera2d::default(),
            Camera {
                order: 10,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            UICamera,
        )
    );
}