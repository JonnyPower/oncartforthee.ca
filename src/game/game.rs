use std::f32::consts::PI;
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::{AmbientLight, Assets, AssetServer, Color, Commands, default, DirectionalLight, light_consts, Mesh, Mesh3d, Meshable, MeshMaterial3d, OnEnter, Plane3d, Plugin, Quat, Res, ResMut, SceneRoot, StandardMaterial, Transform, Vec3};
use crate::state::{InGameState, TitleMenuState};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Playing),
            setup_scene
        );
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));
    let cart = asset_server.load("models/shopping_cart.glb#Scene0");
    commands.spawn((
        SceneRoot(cart),
        Transform::from_xyz(0.0, 0.0, 1.0)
    ));
    // ambient light
    commands.insert_resource(AmbientLight {
        color: ORANGE_RED.into(),
        brightness: 0.2,
    });
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
            .build(),
    ));
}