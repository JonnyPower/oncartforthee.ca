use crate::camera::GameCamera;
use crate::game::animation::{
    setup_animation_graph, AnimationPlugin, AnimationToPlay, PlayerOnStep,
};
use crate::game::effects::hook::PlayerSkillHookPlugin;
use crate::game::effects::particles::ParticlesPlugin;
use crate::game::effects::stomp::PlayerSkillStompPlugin;
use crate::game::effects::vacuum::PlayerSkillVacuumPlugin;
use crate::game::hud::HudPlugin;
use crate::game::item::{ItemIsStomped, ItemPickup, ItemPickupCollider, ItemPickupCountry};
use crate::game::map::map_object::MiscShopObjects;
use crate::game::map::wall::spawn_walls;
use crate::game::movement::{MovementPlugin, MovementSettings};
use crate::game::player::PlayerPlugin;
use crate::state::{InGameState, TitleMenuState};
use bevy::app::App;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::gltf::GltfAssetLabel;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::{
    debug, default, in_state, info, light_consts, Added, AmbientLight, AnimationClip,
    AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, AnimationPlayer, AssetServer, Assets,
    BuildChildren, Camera, ChildBuild, Children, Color, Commands, Component, Dir3,
    DirectionalLight, Entity, EventReader, FixedUpdate, GlobalTransform, Handle, HierarchyQueryExt,
    IntoSystemConfigs, Mesh, Mesh3d, MeshMaterial3d, Meshable, Name, OnEnter, Parent, PbrBundle,
    Plane3d, Plugin, Quat, Query, Res, ResMut, Resource, SceneRoot, Sprite, SpriteBundle,
    StandardMaterial, Transform, Trigger, Update, Vec2, Vec3, With, Without,
};
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::scene::SceneInstanceReady;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::plugin::WriteRapierContext;
use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, CollisionGroups, Damping, ExternalForce, ExternalImpulse,
    GravityScale, Group, KinematicCharacterController, RapierContext, RapierContextSimulation,
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
            (detect_item_landing_floor).run_if(in_state(InGameState::Playing)),
        );
        // app.add_systems(
        //     FixedUpdate,
        //     ().run_if(in_state(InGameState::Playing)),
        // );
        app.add_plugins(PlayerPlugin);
        app.add_plugins(MovementPlugin);
        app.add_plugins(ParticlesPlugin);
        app.add_plugins(PlayerSkillStompPlugin);
        // app.add_plugins(PlayerSkillVacuumPlugin);
        app.add_plugins(PlayerSkillHookPlugin);
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
#[require(TrackedByKDTree, Velocity, ExternalImpulse, GravityScale, RigidBody)]
pub struct American;

#[derive(Component)]
pub struct FloorTag;

#[derive(Resource)]
pub struct ScoreResource {
    pub score: i32,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
            Mesh3d(meshes.add(Plane3d::default().mesh().size(25.0, 82.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(tile_image.clone()),
                uv_transform: Affine2::from_scale(Vec2::new(25.0 / 5.0, 82.0 / 5.0)),
                ..default()
            })),
            FloorTag,
            Transform::from_xyz(-1.5, 0.0, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Collider::cuboid(12.5, 0.01, 41.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_3, Group::GROUP_1 | Group::GROUP_2), // Collision events when items touch floor
            ));
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
                Collider::cuboid(0.5, 0.75, 0.5),
                Transform::from_xyz(0.0, 0.75, 0.0),
            ));
        });
    for i in -4..4 {
        if i == 0 {
            continue;
        }
        for j in -4..4 {
            let shelf = MiscShopObjects::Shelf.spawn(&mut commands, &asset_server);
            commands.entity(shelf).insert(Transform::from_xyz(
                3.0 * i as f32,
                0.0,
                -10.0 * j as f32,
            ));
        }
    }
    spawn_walls(
        &mut commands,
        &asset_server,
        Vec3::new(11.0, 0.0, 41.0),
        Vec3::new(11.0, 0.0, -41.0),
    )
    .expect("failed to create walls");
    spawn_walls(
        &mut commands,
        &asset_server,
        Vec3::new(11.0, 0.0, 41.0),
        Vec3::new(-14.0, 0.0, 41.0),
    )
    .expect("failed to create walls");
    spawn_walls(
        &mut commands,
        &asset_server,
        Vec3::new(-14.0, 0.0, 41.0),
        Vec3::new(-14.0, 0.0, -41.0),
    )
    .expect("failed to create walls");
    spawn_walls(
        &mut commands,
        &asset_server,
        Vec3::new(-14.0, 0.0, -41.0),
        Vec3::new(11.0, 0.0, -41.0),
    )
    .expect("failed to create walls");
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

fn detect_item_landing_floor(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    parents_q: Query<&Parent>,
    items_q: Query<(Entity), With<ItemPickup>>,
    floor_q: Query<(Entity), With<FloorTag>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let mut item_entity = None;
            let mut floor_entity = None;

            for &entity in [entity1, entity2].iter() {
                if let Ok(parent) = parents_q.get(*entity) {
                    if let Ok(item_e) = items_q.get(**parent) {
                        item_entity = Some(item_e);
                    }
                    if let Ok(floor_e) = floor_q.get(**parent) {
                        floor_entity = Some(floor_e);
                    }
                }
            }

            // If an item collided with a floor, remove `ItemIsStomped`
            if let (Some(item), Some(_)) = (item_entity, floor_entity) {
                commands.entity(item).remove::<ItemIsStomped>();
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
