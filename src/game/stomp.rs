use crate::game::game::{CartCollider, FlagForItem, Player, TrackedByKDTree, CART_HEIGHT};
use crate::game::item::{ItemIsStomped, ItemPickup, ItemPickupCountry};
use crate::game::particles::{spawn_particle, ParticleAssets};
use crate::state::InGameState;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::{vec3, Affine2, Vec3};
use bevy::prelude::{
    default, in_state, Added, AssetServer, Assets, Children, Color, Commands, Component,
    FixedUpdate, Handle, HierarchyQueryExt, IntoSystemConfigs, KeyCode, LinearRgba, Mesh, Mesh3d,
    MeshMaterial3d, OnRemove, PbrBundle, Plugin, Query, Res, SceneSpawner, StandardMaterial, Torus,
    Transform, Trigger, Update, Vec3Swizzles, With, Without,
};
use bevy::prelude::{DespawnRecursiveExt, GlobalTransform, Parent, ReflectResource, ResMut};
use bevy::prelude::{Entity, Resource};
use bevy::prelude::{EventReader, Vec2};
use bevy::reflect::Reflect;
use bevy::scene::SceneInstance;
use bevy::utils::info;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::prelude::CollisionEvent::Started;
use bevy_rapier3d::prelude::{Collider, ExternalImpulse, Vect, Velocity};
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::SpatialAccess;
use rand::{rng, Rng};
use std::f32::consts::{PI, TAU};

const GRAVITY: f32 = -9.81;

pub struct PlayerStompPlugin;
impl Plugin for PlayerStompPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_stomp, detect_item_landing_on_cart).run_if(in_state(InGameState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            (
                highlight_stomped_items,
                draw_landing_reticule,
                remove_landing_indicators,
                update_landing_reticule,
            )
                .run_if(in_state(InGameState::Playing)),
        );
        app.add_event::<CollisionEvent>();
        app.add_observer(trigger_stomp_removed);
        app.insert_resource(StompResource {
            stomp_distance: 5.0,
            stomp_away_force: -0.01,
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

#[derive(Component)]
struct ItemForLandingIndicator(Entity);

#[derive(Component)]
struct LandingIndicatorForItem(Entity);

#[derive(Resource)]
pub struct ScoreResource {
    pub(crate) score: i32,
}
impl Default for ScoreResource {
    fn default() -> Self {
        ScoreResource { score: 0 }
    }
}

fn draw_landing_reticule(
    mut commands: Commands,
    item_q: Query<
        (Entity, &Velocity, &Transform, &ItemPickupCountry),
        (With<ItemIsStomped>, Without<LandingIndicatorForItem>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, velocity, transform, item_country) in item_q.iter() {
        if let Some(landing_position) = compute_landing_pos(transform.translation, velocity.linvel)
        {
            let color = item_country.highlight_color();
            let indicator = commands
                .spawn((
                    Mesh3d(meshes.add(Torus::new(0.1, 0.05))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::linear_rgba(color.red, color.green, color.blue, 0.5), // Semi-transparent red
                        ..Default::default()
                    })),
                    Transform::from_translation(landing_position),
                    ItemForLandingIndicator(entity),
                ))
                .id();
            commands
                .entity(entity)
                .insert(LandingIndicatorForItem(indicator));
        }
    }
}

fn update_landing_reticule(
    item_q: Query<(&Velocity, &Transform, &LandingIndicatorForItem), (With<ItemIsStomped>)>,
    mut transform_q: Query<
        &mut Transform,
        (
            Without<LandingIndicatorForItem>,
            With<ItemForLandingIndicator>,
        ),
    >,
) {
    for (item_v, item_t, indicator_link) in item_q.iter() {
        if let Some(landing_pos) = compute_landing_pos(item_t.translation, item_v.linvel) {
            let indicator_e = indicator_link.0;
            if let Ok(mut indicator_t) = transform_q.get_mut(indicator_e) {
                indicator_t.translation = landing_pos;
            }
        }
    }
}

fn compute_landing_pos(initial_position: Vec3, initial_velocity: Vect) -> Option<Vec3> {
    // Compute time until impact (assuming flat ground at y = 0)
    let time_to_land = (-initial_velocity.y
        - (initial_velocity.y.powi(2) - 2.0 * GRAVITY * initial_position.y).sqrt())
        / GRAVITY;

    if time_to_land.is_nan() || time_to_land <= 0.0 {
        None
    } else {
        // Compute landing position
        let landing_x = initial_position.x + initial_velocity.x * time_to_land;
        let landing_z = initial_position.z + initial_velocity.z * time_to_land;
        Some(Vec3::new(landing_x, 0.01, landing_z))
    }
}

// FIXME
// fn compute_required_velocity(start: Vec3, target: Vec3) -> Option<Vec2> {
//     let time_to_land = (-start.y.sqrt() - (2.0 * GRAVITY * (target.y - start.y)).sqrt()) / GRAVITY;
//
//     if time_to_land.is_nan() || time_to_land <= 0.1 {
//         None
//     } else {
//         // Solve for velocity components
//         let required_velocity_xz = (target.xz() - start.xz()) / time_to_land;
//         Some(Vec2::new(required_velocity_xz.x, required_velocity_xz.y))
//     }
// }

fn remove_landing_indicators(
    mut commands: Commands,
    query: Query<(Entity, &LandingIndicatorForItem), Without<ItemIsStomped>>,
) {
    for (item_e, link) in query.iter() {
        if let Some(mut indicator_ec) = commands.get_entity(link.0) {
            indicator_ec.despawn_recursive();
        }
        if let Some(mut item_ec) = commands.get_entity(item_e) {
            item_ec.remove::<LandingIndicatorForItem>();
        }
    }
}

fn detect_item_landing_on_cart(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collider_q: Query<(Entity, Option<&Parent>), With<Collider>>,
    item_q: Query<
        (
            &GlobalTransform,
            &ItemPickupCountry,
            &FlagForItem,
            &LandingIndicatorForItem,
        ),
        With<ItemPickup>,
    >,
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
            if let (
                Some(item),
                Some((item_gt, item_country, flag_link, indicator_link)),
                Some(cart),
                Some(cart_t),
            ) = (item_entity, item_result, cart_entity, cart_t)
            {
                if item_gt.translation().y >= cart_t.translation().y + CART_HEIGHT - 0.1 {
                    commands.entity(flag_link.0).despawn_recursive();
                    commands.entity(indicator_link.0).despawn_recursive();
                    commands.entity(item).despawn_recursive();
                    score_res.score += item_country.scores();
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
    mut item_q: Query<
        (&Transform, &mut ExternalImpulse, &Velocity),
        (Without<Player>, With<ItemPickup>),
    >,
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
                    if let Ok((item_t, mut item_impulse, item_v)) = item_q.get_mut(entity) {
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
                        let mut impulse = Vec3::new(0.0, distance_factored_up_force, 0.0)
                            + (direction * distance_factored_away_force);
                        // FIXME
                        // if stomp_settings.stomp_away_force < 0.0 {
                        //     let new_velocity = item_v.linvel + impulse;
                        //     if let Some(landing_pos) = compute_landing_pos(item_t.translation, new_velocity) {
                        //         if landing_pos.distance(offset_to_cart) > 0.1 {
                        //             if let Some(required_velocity_xy) = compute_required_velocity(item_t.translation + distance_factored_up_force, offset_to_cart) {
                        //                 impulse = Vec3::new(required_velocity_xy.x, distance_factored_up_force, required_velocity_xy.y) - item_v.linvel;
                        //             } else {
                        //                 info!("Could not determine required_velocity");
                        //             }
                        //         } else {
                        //             info!("not overshooting");
                        //         }
                        //     } else {
                        //         info!("Could not determine landing pos");
                        //     }
                        // }
                        item_impulse.impulse += impulse;
                        commands.entity(entity).insert(ItemIsStomped);
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

fn highlight_stomped_items(
    mut commands: Commands,
    mut scene_q: Query<(&SceneInstance, &ItemPickupCountry), Added<ItemIsStomped>>,
    scene_spawner: Res<SceneSpawner>,
    mesh_query: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (scene_instance, item_country) in &scene_q {
        for child_entity in scene_spawner.iter_instance_entities(**scene_instance) {
            if let Ok((mesh_entity, material_handle)) = mesh_query.get(child_entity) {
                if let Some(material) = materials.get_mut(material_handle) {
                    let mut new_material = material.clone();
                    new_material.emissive = item_country.highlight_color();
                    let new_material_handle = materials.add(new_material);
                    commands
                        .entity(mesh_entity)
                        .insert(MeshMaterial3d(new_material_handle));
                }
            }
        }
    }
}

fn trigger_stomp_removed(
    trigger: Trigger<OnRemove, ItemIsStomped>,
    mut scene_q: Query<(Entity, &SceneInstance)>,
    children_q: Query<&Children>,
    scene_spawner: Res<SceneSpawner>,
    mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.entity();
    if let Ok((_scene_entity, scene_instance)) = scene_q.get_mut(entity) {
        for child_entity in scene_spawner.iter_instance_entities(**scene_instance) {
            if let Ok(material_handle) = mesh_query.get(child_entity) {
                if let Some(material) = materials.get_mut(material_handle) {
                    material.emissive = LinearRgba::NONE;
                }
            }
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
