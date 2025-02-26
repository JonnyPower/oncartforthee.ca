use crate::game::animation::{setup_animation_graph, AnimationToPlay};
use crate::game::game::{ScoreResource, TrackedByKDTree};
use crate::game::item::{ItemPickup, ItemPickupCountry};
use crate::game::movement::MovementSettings;
use crate::state::InGameState;
use bevy::app::{App, Update};
use bevy::asset::{AssetServer, Assets};
use bevy::core::Name;
use bevy::gltf::GltfAssetLabel;
use bevy::hierarchy::{DespawnRecursiveExt, Parent};
use bevy::prelude::{
    in_state, AnimationGraph, BuildChildren, ChildBuild, Commands, Component, Entity, EventReader,
    GlobalTransform, IntoSystemConfigs, OnEnter, Plugin, Query, Res, ResMut, SceneRoot, Transform,
    With,
};
use bevy_rapier3d::dynamics::Damping;
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::pipeline::CollisionEvent::Started;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), spawn_player);
        app.add_systems(
            Update,
            (detect_item_landing_on_cart).run_if(in_state(InGameState::Playing)),
        );
    }
}

#[derive(Component)]
#[require(
    Velocity,
    ExternalImpulse,
    ExternalForce,
    GravityScale,
    RigidBody,
    Ccd(Ccd::enabled),
    TrackedByKDTree,
    MovementSettings(player_movement_defaults),
    Damping(player_damping)
)]
pub struct Player;

fn player_movement_defaults() -> MovementSettings {
    let mut player_ms = MovementSettings::default();
    player_ms.speed = 2.0;
    player_ms.max_speed = 25.0;
    player_ms
}

fn player_damping() -> Damping {
    Damping {
        linear_damping: 8.0,
        angular_damping: 1.0,
    }
}

#[derive(Component)]
#[require(
    CollisionGroups(cart_collider_groups),
    ActiveEvents(active_collision_events)
)]
pub struct CartCollider;

fn cart_collider_groups() -> CollisionGroups {
    CollisionGroups::new(
        Group::GROUP_1,
        Group::GROUP_2 | Group::GROUP_3 | Group::GROUP_4,
    )
}

fn active_collision_events() -> ActiveEvents {
    ActiveEvents::COLLISION_EVENTS
}

pub const CART_HEIGHT: f32 = 0.5;

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
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
            let cart_collider = Collider::cuboid(0.5, CART_HEIGHT, 0.75);
            parent.spawn((
                cart_collider,
                Transform::from_xyz(0.0, CART_HEIGHT, -1.25),
                CartCollider,
            ));
            parent.spawn((
                Collider::capsule_y(0.65, 0.25),
                Transform::from_xyz(0.0, 0.9, 0.1),
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
}

fn detect_item_landing_on_cart(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collider_q: Query<(Entity, Option<&Parent>), With<Collider>>,
    item_q: Query<(&GlobalTransform, &ItemPickupCountry), With<ItemPickup>>,
    cart_q: Query<(&GlobalTransform), With<CartCollider>>,
    mut score_res: ResMut<ScoreResource>,
) {
    for event in collision_events.read() {
        if let Started(e1, e2, _flags) = event {
            let mut item_entity = None;
            let mut cart_entity = None;
            let mut item_result = None;
            let mut cart_t = None;
            for &entity in [e1, e2].iter() {
                // If parent of entity that collided is ItemPickup...
                if let Ok((_, Some(parent))) = collider_q.get(*entity) {
                    if let Ok(query_result) = item_q.get(parent.get()) {
                        item_entity = Some(parent.get());
                        item_result = Some(query_result);
                    }
                }
                // If current collided entity is CartCollider
                if let Ok(cart_transform) = cart_q.get(*entity) {
                    cart_entity = Some(entity);
                    cart_t = Some(cart_transform);
                }
            }
            if let (Some(item), Some((item_gt, item_country)), Some(cart), Some(cart_t)) =
                (item_entity, item_result, cart_entity, cart_t)
            {
                if item_gt.translation().y >= cart_t.translation().y + 0.1 {
                    commands.entity(item).despawn_recursive();
                    score_res.score += item_country.scores();
                }
            }
        }
    }
}
