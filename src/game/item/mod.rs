use crate::game::game::TrackedByKDTree;
use bevy::math::Vec3;
use bevy::prelude::Component;
use bevy::utils::default;
use bevy_rapier3d::prelude::*;

mod item_reader;

pub struct ItemDetails {
    pub name: String,
    pub is_canadian: bool,
    pub model: String,
    pub texture: String,
}

#[derive(Component)]
#[require(TrackedByKDTree, Velocity, ExternalImpulse, GravityScale, RigidBody)]
pub struct ItemPickup;

#[derive(Component)]
#[require(
    ColliderMassProperties(item_pickup_mass),
    CollisionGroups(item_pickup_collision_groups)
)]
pub struct ItemPickupCollider;

fn item_pickup_mass() -> ColliderMassProperties {
    ColliderMassProperties::Mass(0.005)
}

fn item_pickup_collision_groups() -> CollisionGroups {
    CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2)
}
