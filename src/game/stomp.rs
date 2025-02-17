use crate::game::game::{CartCollider, FlagForItem, Player, TrackedByKDTree, CART_HEIGHT};
use crate::game::item::{ItemPickup, ItemPickupCountry};
use crate::game::particles::{spawn_particle, ParticleAssets};
use crate::state::InGameState;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::{vec3, Vec3};
use bevy::prelude::EventReader;
use bevy::prelude::{
    in_state, Commands, IntoSystemConfigs, KeyCode, Plugin, Query, Res, Transform, Update, With,
    Without,
};
use bevy::prelude::{DespawnRecursiveExt, GlobalTransform, Parent, ReflectResource, ResMut};
use bevy::prelude::{Entity, Resource};
use bevy::reflect::Reflect;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::prelude::CollisionEvent::Started;
use bevy_rapier3d::prelude::{Collider, ExternalImpulse};
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::SpatialAccess;
use rand::{rng, Rng};
use std::f32::consts::{PI, TAU};

pub struct PlayerStompPlugin;
impl Plugin for PlayerStompPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_stomp, detect_item_landing_on_cart).run_if(in_state(InGameState::Playing)),
        );
        app.add_event::<CollisionEvent>();
        app.insert_resource(StompResource {
            stomp_distance: 3.0,
            stomp_away_force: 0.0,
            stomp_up_force: 0.08,
            stomp_particles: 80,
            stomp_distance_falloff: 0.5,
        });
        app.register_type::<StompResource>();
        app.init_resource::<ScoreResource>();
        app.add_plugins(ResourceInspectorPlugin::<StompResource>::default());
    }
}

#[derive(Resource, InspectorOptions, Reflect)]
#[reflect(Resource, InspectorOptions)]
pub struct StompResource {
    stomp_distance: f32,
    stomp_away_force: f32,
    stomp_up_force: f32,
    stomp_particles: i32,
    stomp_distance_falloff: f32,
}

#[derive(Resource)]
pub struct ScoreResource {
    pub(crate) score: i32,
}
impl Default for ScoreResource {
    fn default() -> Self {
        ScoreResource { score: 0 }
    }
}

fn detect_item_landing_on_cart(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collider_q: Query<(Entity, Option<&Parent>), With<Collider>>,
    item_q: Query<(&GlobalTransform, &ItemPickupCountry, &FlagForItem), With<ItemPickup>>,
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
                    if let Ok(item_transform) = item_q.get(parent.get()) {
                        item_entity = Some(parent.get());
                        item_result = Some(item_transform);
                    }
                }
                // If current collided entity is CartCollider
                if let Ok(cart_transform) = cart_q.get(*entity) {
                    cart_entity = Some(entity);
                    cart_t = Some(cart_transform);
                }
            }
            if let (
                Some(item),
                Some((item_gt, item_country, item_flag)),
                Some(cart),
                Some(cart_t),
            ) = (item_entity, item_result, cart_entity, cart_t)
            {
                if item_gt.translation().y >= cart_t.translation().y + CART_HEIGHT - 0.1 {
                    commands.entity(item_flag.0).despawn_recursive();
                    commands.entity(item).despawn_recursive();
                    match item_country {
                        ItemPickupCountry::USA => {
                            score_res.score -= 10;
                        }
                        ItemPickupCountry::CA => {
                            score_res.score += 10;
                        }
                        ItemPickupCountry::Mexico => {
                            score_res.score += 5;
                        }
                        ItemPickupCountry::EU => {
                            score_res.score += 2;
                        }
                        ItemPickupCountry::UK => {
                            score_res.score += 3;
                        }
                        ItemPickupCountry::China => {
                            score_res.score -= 1;
                        }
                    }
                }
            }
        }
    }
}

fn handle_stomp(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&Transform, With<Player>>,
    tree: Res<KDTree3<TrackedByKDTree>>,
    mut item_q: Query<(&Transform, &mut ExternalImpulse), (Without<Player>, With<ItemPickup>)>,
    mut american_q: Query<
        (&Transform, &mut ExternalImpulse),
        (Without<Player>, Without<ItemPickup>),
    >,
    particle: Res<ParticleAssets>,
    stomp_settings: Res<StompResource>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Ok(player_t) = player_q.get_single_mut() {
            for (pos, opt_entity) in
                tree.within_distance(player_t.translation, stomp_settings.stomp_distance)
            {
                if let Some(entity) = opt_entity {
                    if let Ok((item_t, mut item_impulse)) = item_q.get_mut(entity) {
                        let offset_to_cart = player_t.translation + Vec3::new(0.0, 0.0, -1.3);
                        let stomp_distance = item_t.translation.distance(player_t.translation);
                        let distance_factor = (1.0
                            - (stomp_distance / stomp_settings.stomp_distance))
                            .powf(stomp_settings.stomp_distance_falloff)
                            .clamp(0.0, 1.0);

                        let direction = (item_t.translation - offset_to_cart).normalize_or_zero();
                        let distance_factored_up_force =
                            stomp_settings.stomp_up_force * distance_factor;
                        let distance_factored_away_force =
                            stomp_settings.stomp_away_force * distance_factor;

                        item_impulse.impulse += Vec3::new(0.0, distance_factored_up_force, 0.0)
                            + (direction * distance_factored_away_force);
                    }
                    if let Ok((american_t, mut american_impulse)) = american_q.get_mut(entity) {
                        let direction =
                            (american_t.translation - player_t.translation).normalize_or_zero();
                        american_impulse.impulse +=
                            direction * stomp_settings.stomp_away_force * 100.0;
                    }
                }
            }
            draw_stomp_particles(
                &mut commands,
                &player_t,
                &particle,
                stomp_settings.stomp_particles,
            );
        }
    }
}

fn draw_stomp_particles(
    mut commands: &mut Commands,
    player_t: &Transform,
    particle: &Res<ParticleAssets>,
    stomp_particles: i32,
) {
    let mut rng = rng();
    for i in 1..stomp_particles {
        let size = rng.random_range(0.01..0.03);
        let particle_spawn = player_t.translation;
        let theta = rng.random_range(0.0..TAU); // 0 to 2π
        let phi = rng.random_range(0.0..PI); // 0 to π
        let x = phi.sin() * theta.cos();
        let y = rng.random_range(0.1..0.5);
        let z = phi.sin() * theta.sin();
        let direction = Vec3::new(x, y, z) * 20.0;
        commands.queue(spawn_particle(
            particle.mesh.clone(),
            particle.material.clone(),
            particle_spawn.reject_from_normalized(Vec3::Y),
            rng.random_range(0.5..1.5),
            size,
            direction,
        ));
    }
}
