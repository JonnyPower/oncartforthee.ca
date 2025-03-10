use crate::game::item::{ItemPickup, ItemPickupCollider};
use crate::game::map::{Category, ShopObject, ShopObjectScene};
use bevy::asset::AssetServer;
use bevy::core::Name;
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::prelude::{
    Bundle, Children, Commands, Entity, HierarchyQueryExt, Query, Res, SceneRoot, Transform,
    Trigger, With, Without,
};
use bevy::scene::SceneInstanceReady;
use bevy_rapier3d::geometry::Group;
use bevy_rapier3d::prelude::{Collider, CollisionGroups};

pub struct MiscShelf;

impl ShopObject for MiscShelf {
    fn categories(&self) -> Vec<(f32, Category)> {
        vec![(0.5, Category::Snacks), (0.5, Category::Condiments)]
    }

    fn colliders_with_transforms(&self) -> Vec<impl Bundle> {
        vec![
            (
                Collider::cuboid(0.035, 0.94, 0.0475),
                Transform::from_xyz(-1.43, 0.94, 0.395),
            ),
            (
                Collider::cuboid(0.035, 0.94, 0.0475),
                Transform::from_xyz(-1.43, 0.94, -0.395),
            ),
            (
                Collider::cuboid(0.035, 0.94, 0.0475),
                Transform::from_xyz(1.43, 0.94, 0.395),
            ),
            (
                Collider::cuboid(0.035, 0.94, 0.0475),
                Transform::from_xyz(1.43, 0.94, -0.395),
            ),
            (
                Collider::cuboid(1.4, 0.025, 0.45),
                Transform::from_xyz(0.0, 0.375, 0.0),
            ),
            (
                Collider::cuboid(1.4, 0.025, 0.45),
                Transform::from_xyz(0.0, 1.075, 0.0),
            ),
            (
                Collider::cuboid(1.4, 0.025, 0.45),
                Transform::from_xyz(0.0, 1.74, 0.0),
            ),
        ]
    }
    fn player_collider(&self) -> impl Bundle {
        (
            Collider::cuboid(1.5, 0.95, 0.55),
            Transform::from_xyz(0.0, 0.95, 0.0),
            CollisionGroups::new(Group::GROUP_4, Group::GROUP_1),
        )
    }
    fn path(&self) -> &str {
        "models/SM_Prop_Shop_Shelf_Basic_01.glb#Scene0"
    }
}
