use crate::game::game::TrackedByKDTree;
use bevy::math::Vec3;
use bevy::prelude::{Component, LinearRgba};
use bevy::utils::default;
use bevy_rapier3d::prelude::*;
use rand::distr::StandardUniform;
use rand::prelude::Distribution;
use rand::Rng;

mod item_reader;

#[derive(Component)]
#[require(
    TrackedByKDTree,
    Velocity,
    ExternalImpulse,
    GravityScale,
    RigidBody,
    ItemPickupCountry(item_origin_random)
)]
pub struct ItemPickup;

#[derive(Component)]
#[require(
    ColliderMassProperties(item_pickup_mass),
    CollisionGroups(item_pickup_collision_groups)
)]
pub struct ItemPickupCollider;

#[derive(Component)]
pub struct ItemIsStomped;

#[derive(Component)]
pub enum ItemPickupCountry {
    USA,
    CA,
    Mexico,
    EU,
    UK,
    China,
}
impl ItemPickupCountry {
    pub fn asset_path(&self) -> &'static str {
        match self {
            ItemPickupCountry::USA => "images/fl_us.png",
            ItemPickupCountry::CA => "images/fl_ca.png",
            ItemPickupCountry::Mexico => "images/fl_mx.png",
            ItemPickupCountry::EU => "images/fl_eu.png",
            ItemPickupCountry::UK => "images/fl_uk.png",
            ItemPickupCountry::China => "images/fl_cn.png",
        }
    }
    pub fn highlight_color(&self) -> LinearRgba {
        if self.scores() > 0 {
            LinearRgba::rgb(0.0, 0.1, 0.0)
        } else {
            LinearRgba::rgb(0.1, 0.0, 0.0)
        }
    }
    pub fn scores(&self) -> i32 {
        match self {
            ItemPickupCountry::USA => -10,
            ItemPickupCountry::CA => 10,
            ItemPickupCountry::Mexico => 5,
            ItemPickupCountry::EU => 2,
            ItemPickupCountry::UK => 3,
            ItemPickupCountry::China => -1,
        }
    }
}
impl Distribution<ItemPickupCountry> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ItemPickupCountry {
        match rng.random_range(0..=5) {
            0 => ItemPickupCountry::USA,
            1 => ItemPickupCountry::CA,
            2 => ItemPickupCountry::Mexico,
            3 => ItemPickupCountry::EU,
            4 => ItemPickupCountry::UK,
            5 => ItemPickupCountry::China,
            _ => unreachable!(),
        }
    }
}

fn item_pickup_mass() -> ColliderMassProperties {
    ColliderMassProperties::Mass(0.005)
}

fn item_pickup_collision_groups() -> CollisionGroups {
    CollisionGroups::new(
        Group::GROUP_2,
        Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
    )
}

fn item_origin_random() -> ItemPickupCountry {
    rand::random()
}
