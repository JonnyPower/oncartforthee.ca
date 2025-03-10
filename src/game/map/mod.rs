use crate::game::item::{ItemPickup, ItemPickupCollider};
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::core::Name;
use bevy::hierarchy::{BuildChildren, ChildBuild, Children, HierarchyQueryExt};
use bevy::prelude::{
    Bundle, Commands, Component, Entity, OnAdd, Plugin, Query, Res, SceneRoot, Transform, Trigger,
    With, Without,
};
use bevy::scene::SceneInstanceReady;
use bevy_rapier3d::geometry::Collider;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;

pub mod misc_shelf;
pub mod wall;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_spawned_shop_object_observe_scene_ready);
    }
}

#[derive(Component)]
pub struct ShopObjectScene;

#[derive(PartialEq)]
pub enum Category {
    Bakery,
    Produce,
    Dairy,
    Meat,
    Canned,
    Snacks,
    Beverages,
    Frozen,
    Condiments,
}

#[derive(Component)]
pub struct CategoryDistribution(pub Vec<(f32, Category)>);

pub trait ShopObject {
    fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
        let mut obj = commands.spawn(self.scene(&asset_server));
        obj.insert(ShopObjectScene);
        obj.insert(CategoryDistribution(self.categories()));
        obj.with_children(|parent| {
            for collider_with_transform in self.colliders_with_transforms() {
                parent.spawn(collider_with_transform);
            }
            parent.spawn(self.player_collider());
        });
        obj.id()
    }
    fn scene(&self, asset_server: &Res<AssetServer>) -> SceneRoot {
        let scene = asset_server.load(self.path());
        SceneRoot(scene)
    }
    fn categories(&self) -> Vec<(f32, Category)>;
    fn colliders_with_transforms(&self) -> Vec<impl Bundle>;
    fn player_collider(&self) -> impl Bundle;
    fn path(&self) -> &str;
}

fn on_spawned_shop_object_observe_scene_ready(
    trigger: Trigger<OnAdd, ShopObjectScene>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity())
        .observe(on_shop_object_finish);
}

fn on_shop_object_finish(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    child_q: Query<&Children>,
    name_t_q: Query<(&Name, &Transform), Without<SceneRoot>>,
    t_q: Query<(&Transform, &CategoryDistribution), With<SceneRoot>>,
) {
    let burger = asset_server.load("models/burger.glb#Scene0");
    let (parent_t, category_dist) = t_q.get(trigger.entity()).unwrap();
    let category_weights: Vec<f32> = category_dist.0.iter().map(|(weight, _)| *weight).collect();
    let dist = WeightedIndex::new(&category_weights).unwrap();
    let mut rng = rand::rng();
    for child in child_q.iter_descendants(trigger.entity()) {
        if let Ok((name, t)) = name_t_q.get(child) {
            if name.as_str().starts_with("Item") {
                let item_category = &category_dist.0[dist.sample(&mut rng)].1;
                // Fixme: distribute items based on category
                if *item_category == Category::Snacks {
                    let mut ec = commands.spawn((
                        Name::new("Burger"),
                        SceneRoot(burger.clone()),
                        Transform::from_translation(t.translation + parent_t.translation),
                        ItemPickup,
                    ));
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
}
