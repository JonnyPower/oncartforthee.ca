use crate::game::movement::{MovementPlugin, MovementSettings};
use crate::state::{InGameState, TitleMenuState};
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::gltf::GltfAssetLabel;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::{
    debug, default, info, light_consts, AmbientLight, AnimationGraph, AnimationGraphHandle,
    AnimationNodeIndex, AnimationPlayer, AssetServer, Assets, BuildChildren, ChildBuild, Children,
    Color, Commands, Component, DirectionalLight, Handle, HierarchyQueryExt, Mesh, Mesh3d,
    MeshMaterial3d, Meshable, Name, OnEnter, Plane3d, Plugin, Quat, Query, Res, ResMut, SceneRoot,
    StandardMaterial, Transform, Trigger, Vec2, Vec3,
};
use bevy::scene::SceneInstanceReady;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), setup_scene);
        app.add_plugins(MovementPlugin);
    }
}

#[derive(Component)]
#[require(MovementSettings, Velocity, ExternalImpulse, GravityScale, RigidBody)]
pub struct Player;

#[derive(Component)]
pub struct AnimationToPlay {
    pub graph_handle: Handle<AnimationGraph>,
    pub index: AnimationNodeIndex,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("scene setup");
    // ground plane
    let tile_image = asset_server.load_with_settings("textures/tile.png", |s: &mut _| {
        *s = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                // rewriting mode to repeat image,
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    });
    commands
        .spawn((
            Name::new("Floor"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(300.0, 300.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(tile_image.clone()),
                uv_transform: Affine2::from_scale(Vec2::new(200., 200.)),
                ..default()
            })),
        ))
        .with_children(|parent| {
            parent.spawn((
                Collider::cuboid(150.0, 0.1, 150.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
    let cart = asset_server.load("models/shopping_cart.glb#Scene0");
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/shopping_cart.glb")),
    );
    let graph_handle = graphs.add(graph);
    commands
        .spawn((
            Name::new("Player"),
            SceneRoot(cart),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Player,
            MovementSettings {
                speed: 4.0,
                max_speed: 50.0,
            },
            Damping {
                linear_damping: 8.0,
                angular_damping: 1.0,
            },
            AnimationToPlay {
                graph_handle,
                index,
            },
        ))
        .observe(setup_animation_graph)
        .with_children(|parent| {
            parent.spawn((
                Collider::cuboid(0.5, 0.5, 1.0),
                Transform::from_xyz(0.0, 0.5, 0.0),
            ));
        });
    let plant = asset_server.load("models/plant.glb#Scene0");
    commands
        .spawn((
            Name::new("Plant"),
            SceneRoot(plant),
            Transform::from_xyz(2.0, 0.0, 1.0),
        ))
        .with_children(|parent| {
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

fn setup_animation_graph(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(trigger.entity()) {
        for child in children.iter_descendants(trigger.entity()) {
            if let Ok(mut player) = players.get_mut(child) {
                player.stop_all();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}
