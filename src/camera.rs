use crate::game::player::Player;
use crate::state::InGameState;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{
    default, in_state, App, ButtonInput, Camera, Camera2d, Camera3d, ClearColorConfig, Commands,
    Component, EventReader, IntoSystemConfigs, MouseButton, Plugin, Quat, Query, Res, ResMut,
    Resource, Startup, Time, Transform, Update, Vec3, With, Without,
};

#[derive(Component)]
pub struct UICamera;

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource)]
pub struct PlayerCameraOffset {
    yaw: f32,
    pitch: f32,
    distance: f32,
}
impl Default for PlayerCameraOffset {
    fn default() -> Self {
        PlayerCameraOffset {
            yaw: 0.0,
            pitch: 0.0,
            distance: 5.0,
        }
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(
            Update,
            (update_camera_offset, follow_player_with_offsets)
                .run_if(in_state(InGameState::Playing)),
        );
        app.insert_resource(PlayerCameraOffset::default());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::from_xyz(5.0, 15.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        UICamera,
    ));
}

fn follow_player_with_offsets(
    player_q: Query<&Transform, (Without<GameCamera>, With<Player>)>,
    mut camera_q: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
    camera_offset: Res<PlayerCameraOffset>,
    time: Res<Time>,
) {
    if let Ok(player_t) = player_q.get_single() {
        let mut camera_t = camera_q.single_mut();

        // Compute player's forward direction
        let player_forward = player_t.rotation * Vec3::Z; // Player's local +Z direction

        // Compute base offset in player's local space (behind and above the player)
        let base_offset = Vec3::new(0.0, 3.0, camera_offset.distance);

        // Compute the rotation offset relative to the player's rotation
        let relative_rotation = Quat::from_axis_angle(Vec3::Y, camera_offset.yaw)
            * Quat::from_axis_angle(Vec3::X, camera_offset.pitch);

        // Apply both the player's rotation and the user's camera rotation
        let final_offset = player_t.rotation * (relative_rotation * base_offset);

        // Compute target camera position
        let target_position = player_t.translation + final_offset;

        // Smoothly interpolate camera movement
        camera_t.translation = camera_t
            .translation
            .lerp(target_position, 1.0 - (-time.delta_secs() * 5.0).exp());

        // Make the camera look at the player (slightly above for better framing)
        camera_t.look_at(player_t.translation + Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}

fn update_camera_offset(
    mut camera_offset: ResMut<PlayerCameraOffset>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    if mouse_input.pressed(MouseButton::Right) {
        for event in mouse_motion_events.read() {
            camera_offset.yaw -= event.delta.x * 0.005;
            camera_offset.pitch = (camera_offset.pitch - event.delta.y * 0.005).clamp(
                -std::f32::consts::FRAC_PI_2 + 0.1,
                std::f32::consts::FRAC_PI_2 - 0.1,
            );
        }
    }

    // Adjust zoom based on scroll wheel
    for event in scroll_events.read() {
        camera_offset.distance = (camera_offset.distance - event.y * 0.5).clamp(2.0, 10.0);
    }
}
