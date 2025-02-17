use crate::camera::GameCamera;
use crate::game::animation::{setup_animation_graph, AnimationPlugin, AnimationToPlay};
use crate::game::hud::HudPlugin;
use crate::game::item::{ItemPickup, ItemPickupCollider, ItemPickupCountry};
use crate::game::movement::{MovementPlugin, MovementSettings};
use crate::game::particles::ParticlesPlugin;
use crate::game::stomp::PlayerStompPlugin;
use crate::state::{InGameState, TitleMenuState};
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::gltf::GltfAssetLabel;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::{
    debug, default, in_state, info, light_consts, Added, AmbientLight, AnimationGraph,
    AnimationGraphHandle, AnimationNodeIndex, AnimationPlayer, AssetServer, Assets, BuildChildren,
    Camera, ChildBuild, Children, Color, Commands, Component, Dir3, DirectionalLight, Entity,
    GlobalTransform, Handle, HierarchyQueryExt, IntoSystemConfigs, Mesh, Mesh3d, MeshMaterial3d,
    Meshable, Name, OnEnter, PbrBundle, Plane3d, Plugin, Quat, Query, Res, ResMut, SceneRoot,
    Sprite, SpriteBundle, StandardMaterial, Transform, Trigger, Update, Vec2, Vec3, With, Without,
};
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::scene::SceneInstanceReady;
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, CollisionGroups, Damping, ExternalImpulse, GravityScale, Group,
    RigidBody, Velocity,
};
use bevy_rapier3d::rapier::prelude::{ColliderBuilder, InteractionGroups};
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use std::f32::consts::PI;
use std::time::Duration;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), setup_scene);
        app.add_systems(
            Update,
            (add_item_origin_flag, keep_flag_facing_camera).run_if(in_state(InGameState::Playing)),
        );
        app.add_plugins(MovementPlugin);
        app.add_plugins(ParticlesPlugin);
        app.add_plugins(PlayerStompPlugin);
        app.add_plugins(HudPlugin);
        app.add_plugins(AnimationPlugin);
        app.add_plugins(
            AutomaticUpdate::<TrackedByKDTree>::new().with_spatial_ds(SpatialStructure::KDTree3),
        );
    }
}

#[derive(Component, Default)]
pub struct TrackedByKDTree;

#[derive(Component)]
#[require(
    Velocity,
    ExternalImpulse,
    GravityScale,
    RigidBody,
    TrackedByKDTree,
    MovementSettings(player_movement_defaults),
    Damping(player_damping)
)]
pub struct Player;

fn player_movement_defaults() -> MovementSettings {
    MovementSettings {
        speed: 2.0,
        max_speed: 25.0,
    }
}

fn player_damping() -> Damping {
    Damping {
        linear_damping: 8.0,
        angular_damping: 1.0,
    }
}

#[derive(Component)]
#[require(TrackedByKDTree, Velocity, ExternalImpulse, GravityScale, RigidBody)]
pub struct American;

#[derive(Component)]
#[require(
    CollisionGroups(cart_collider_groups),
    ActiveEvents(active_collision_events)
)]
pub struct CartCollider;

#[derive(Component)]
pub struct ItemForFlag(pub Entity);

#[derive(Component)]
pub struct FlagForItem(pub Entity);

fn cart_collider_groups() -> CollisionGroups {
    CollisionGroups::new(Group::GROUP_1, Group::GROUP_2)
}

fn active_collision_events() -> ActiveEvents {
    ActiveEvents::COLLISION_EVENTS
}

pub const CART_HEIGHT: f32 = 0.5;

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
            Transform::from_xyz(0.0, 0.0, 0.0),
            Player,
        ))
        .with_children(|parent| {
            // Cart Collider
            let cart_collider = Collider::cuboid(0.5, 0.5, 0.75);
            parent.spawn((
                cart_collider,
                Transform::from_xyz(0.0, CART_HEIGHT, -1.25),
                CartCollider,
            ));
            parent
                .spawn((
                    SceneRoot(cart),
                    Transform::from_xyz(0.0, 0.0, -0.75),
                    AnimationToPlay {
                        graph_handle,
                        index,
                    },
                ))
                .observe(setup_animation_graph);
        });
    let syrup = asset_server.load("models/syrup.glb#Scene0");
    commands
        .spawn((
            Name::new("Syrup"),
            SceneRoot(syrup),
            ItemPickup,
            Transform::from_xyz(-2.0, 0.0, -2.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Collider::cuboid(0.1, 0.3, 0.1),
                Transform::from_xyz(0.0, 0.3, 0.0),
                ItemPickupCollider,
            ));
        });
    let america = asset_server.load("models/american.glb#Scene0");
    commands
        .spawn((
            Name::new("American"),
            SceneRoot(america),
            Transform::from_xyz(2.0, 0.0, 2.0),
            Damping {
                linear_damping: 1.5,
                angular_damping: 1.0,
            },
            American,
        ))
        .with_children(|parent| {
            parent.spawn((
                Collider::cuboid(0.5, 1.0, 0.5),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ));
        })
        .observe(setup_ragdoll);
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
    for i in 0..100 {
        let burger = asset_server.load("models/burger.glb#Scene0");
        commands
            .spawn((
                Name::new("Burger"),
                SceneRoot(burger),
                Transform::from_xyz(-2.0, 1.0 * i as f32, -1.0),
                ItemPickup,
            ))
            .with_children(|parent| {
                parent.spawn((Collider::cuboid(0.1, 0.1, 0.1), ItemPickupCollider));
            });
    }
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

fn add_item_origin_flag(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut country_q: Query<(Entity, &Transform, &ItemPickupCountry), Added<ItemPickupCountry>>,
    mut camera_q: Query<&Transform, With<GameCamera>>,
) {
    if let Ok(camera_t) = camera_q.get_single() {
        for (entity, item_t, country) in country_q.iter_mut() {
            let flag_e = commands
                .spawn((
                    Mesh3d(
                        meshes.add(
                            Plane3d {
                                normal: Dir3::Z,
                                ..default()
                            }
                            .mesh()
                            .size(0.2, 0.15),
                        ),
                    ),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load(country.asset_path())),
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from(*item_t).looking_at(camera_t.translation, Vec3::Y),
                    ItemForFlag(entity),
                ))
                .id();
            commands.entity(entity).insert(FlagForItem(flag_e));
        }
    }
}

fn keep_flag_facing_camera(
    mut commands: Commands,
    mut country_q: Query<(&mut Transform, &ItemForFlag)>,
    mut transform_q: Query<&mut Transform, (Without<ItemForFlag>, Without<GameCamera>)>,
    camera_q: Query<&GlobalTransform, (With<GameCamera>, Without<ItemForFlag>)>,
) {
    if let Ok(camera_gt) = camera_q.get_single() {
        for (mut flag_t, target) in country_q.iter_mut() {
            if let Ok(target_t) = transform_q.get(target.0) {
                let camera_world_pos = camera_gt.translation();
                flag_t.translation = target_t.translation;
                flag_t.look_at(camera_world_pos, Vec3::Y);
            }
        }
    }
}

fn setup_ragdoll(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    skeleton_query: Query<&Children, With<SkinnedMesh>>,
) {
}
