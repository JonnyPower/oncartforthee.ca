use std::f32::consts::PI;
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::{AmbientLight, Assets, AssetServer, Color, Commands, default, DirectionalLight, light_consts, Mesh, Mesh3d, Meshable, MeshMaterial3d, OnEnter, Plane3d, Plugin, Quat, Res, ResMut, SceneRoot, StandardMaterial, Transform, Vec3, Component, debug, info, Vec2};
use crate::game::movement::{MovementPlugin, MovementSpeed};
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
#[require(MovementSpeed)]
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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(300.0, 300.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(tile_image.clone()),
            uv_transform: Affine2::from_scale(Vec2::new(100., 100.)),
            ..default()
        })),
    ));
    let cart = asset_server.load("models/shopping_cart.glb#Scene0");
    commands.spawn((
        SceneRoot(cart),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
        MovementSpeed(10.0)
    ));
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