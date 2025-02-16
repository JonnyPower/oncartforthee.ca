use std::f32::consts::PI;
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy_rapier3d::prelude::*;
use bevy::prelude::{AmbientLight, Assets, AssetServer, Color, Commands, default, DirectionalLight, light_consts, Mesh, Mesh3d, Meshable, MeshMaterial3d, OnEnter, Plane3d, Plugin, Quat, Res, ResMut, SceneRoot, StandardMaterial, Transform, Vec3, Component, debug, info, Vec2, BuildChildren, ChildBuild, Name};
use crate::game::movement::{MovementPlugin, MovementSettings};
use crate::state::{InGameState, TitleMenuState};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Playing),
            setup_scene
        );
        app.add_plugins(MovementPlugin);
    }
}

#[derive(Component)]
#[require(MovementSettings, Velocity, ExternalImpulse, GravityScale, RigidBody)]
pub struct Player;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    info!("scene setup");
    // ground plane
    let tile_image = asset_server.load_with_settings(
        "textures/tile.png",
        |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        }
    );
    commands.spawn((
        Name::new("Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(300.0, 300.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(tile_image.clone()),
            uv_transform: Affine2::from_scale(Vec2::new(200., 200.)),
            ..default()
        }))
    )).with_children(|parent| {
        parent.spawn((
            Collider::cuboid(150.0, 0.1, 150.0),
            Transform::from_xyz(0.0, 0.0, 0.0)
        ));
    });
    let cart = asset_server.load("models/shopping_cart.glb#Scene0");
    commands.spawn((
        Name::new("Player"),
        SceneRoot(cart),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
        MovementSettings {
            speed: 125.0,
            max_speed: 350.0
        },
        Damping {
            linear_damping: 5.0,
            angular_damping: 1.0,
        }
    )).with_children(|parent| {
        parent.spawn((
            Collider::cuboid(0.5, 0.5, 1.0),
            Transform::from_xyz(0.0, 0.5, -0.5),
        ));
    });
    let plant = asset_server.load("models/plant.glb#Scene0");
    commands.spawn((
        Name::new("Plant"),
        SceneRoot(plant),
        Transform::from_xyz(2.0, 0.0, 1.0),
    )).with_children(|parent| {
        parent.spawn((
            Collider::cuboid(0.5, 3.0, 0.5),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 2_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));
}