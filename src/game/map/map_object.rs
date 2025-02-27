use crate::game::effects::hook::hook_item_on_click;
use crate::game::item::{ItemPickup, ItemPickupCollider};
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

pub enum MiscShopObjects {
    Shelf,
}

impl MiscShopObjects {
    pub fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
        let mut obj = commands.spawn(self.scene(&asset_server));
        obj.with_children(|parent| {
            for collider_with_transform in self.colliders_with_transforms() {
                parent.spawn(collider_with_transform);
            }
            parent.spawn(self.player_collider());
        });
        obj.observe(on_scene_finish);
        obj.id()
    }
    // get item slots fn
    // spawn item per slot? as child or standalone?
    fn colliders_with_transforms(&self) -> Vec<impl Bundle> {
        match self {
            MiscShopObjects::Shelf => {
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
        }
    }
    fn player_collider(&self) -> impl Bundle {
        (
            Collider::cuboid(1.5, 0.95, 0.55),
            Transform::from_xyz(0.0, 0.95, 0.0),
            CollisionGroups::new(Group::GROUP_4, Group::GROUP_1),
        )
    }
    fn scene(&self, asset_server: &Res<AssetServer>) -> SceneRoot {
        let scene = asset_server.load(self.path());
        SceneRoot(scene)
    }
    fn path(&self) -> &str {
        match self {
            Self::Shelf => "models/SM_Prop_Shop_Shelf_Basic_01.glb#Scene0",
        }
    }
}

fn on_scene_finish(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    child_q: Query<&Children>,
    name_t_q: Query<(&Name, &Transform), Without<SceneRoot>>,
    t_q: Query<&Transform, With<SceneRoot>>,
) {
    let burger = asset_server.load("models/burger.glb#Scene0");
    let parent_t = t_q.get(trigger.entity()).unwrap();
    for child in child_q.iter_descendants(trigger.entity()) {
        if let Ok((name, t)) = name_t_q.get(child) {
            if name.as_str().starts_with("Item") {
                let mut ec = commands.spawn((
                    Name::new("Burger"),
                    SceneRoot(burger.clone()),
                    Transform::from_translation(t.translation + parent_t.translation),
                    ItemPickup,
                ));
                ec.observe(hook_item_on_click);
                ec.with_children(|parent| {
                    parent.spawn((
                        Collider::cuboid(0.1, 0.1, 0.1),
                        Transform::from_xyz(0.0, 0.1, 0.0),
                        ItemPickupCollider,
                    ));
                });
            }
        }
    }
}
