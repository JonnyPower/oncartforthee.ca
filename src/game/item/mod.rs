use crate::game::game::TrackedByKDTree;
use bevy::prelude::Component;
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
