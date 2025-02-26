use crate::camera::GameCamera;
use crate::game::effects::particles::{spawn_particle, spawn_particle_t};
use crate::game::effects::stomp::StompParticleAssets;
use crate::game::game::TrackedByKDTree;
use crate::game::item::{item_pickup_collision_groups, ItemPickup};
use crate::game::player::Player;
use crate::hierarchy::get_root_parent_entity;
use bevy::app::App;
use bevy::asset::{Assets, Handle};
use bevy::color::palettes::basic::WHITE;
use bevy::hierarchy::Parent;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Camera, Color, GlobalTransform, Quat, Real, ReflectResource, Window};
use bevy::prelude::{Commands, Cylinder, Dir3, FromWorld, Mesh, Sphere, World};
use bevy::prelude::{
    MouseButton, Plugin, Query, Reflect, Res, Resource, Transform, Update, With, Without,
};
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::dynamics::{ExternalImpulse, Velocity};
use bevy_rapier3d::geometry::Group;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, QueryFilter, ReadRapierContext};
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::SpatialAccess;
use rand::{rng, Rng};
use web_sys::js_sys::Math;

pub struct PlayerSkillVacuumPlugin;
impl Plugin for PlayerSkillVacuumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_click);
        app.insert_resource(VacuumResource {
            suck_distance: 10.0,
            suck_to_force: 0.005,
        });
        app.register_type::<VacuumResource>();
        app.add_plugins(ResourceInspectorPlugin::<VacuumResource>::default());
        app.init_resource::<VacuumParticleAssets>();
    }
}

#[derive(Resource, InspectorOptions, Reflect)]
#[reflect(Resource, InspectorOptions)]
pub struct VacuumResource {
    suck_distance: f32,
    suck_to_force: f32,
}

#[derive(Resource)]
pub struct VacuumParticleAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}
impl FromWorld for VacuumParticleAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Cylinder::new(0.01, 0.05)),
            material: world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgba(0.5, 0.5, 0.5, 0.5),
                    unlit: true,
                    ..Default::default()
                }),
        }
    }
}

fn handle_click(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    tree: Res<KDTree3<TrackedByKDTree>>,
    mut player_q: Query<&Transform, With<Player>>,
    mut item_q: Query<
        (&Transform, &mut ExternalImpulse, &Velocity),
        (Without<Player>, With<ItemPickup>),
    >,
    parent_query: Query<&Parent>,
    rapier_context_q: ReadRapierContext,
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    vacuum_settings: Res<VacuumResource>,
    particle_assets: Res<VacuumParticleAssets>,
) {
    // if mouse_input.pressed(MouseButton::Left) {
    //     if let Ok(player_t) = player_q.get_single_mut() {
    //         let forward_offset = Vec3::new(0.0, 0.75, -1.3);
    //         let rotated_offset = player_t.rotation * forward_offset;
    //         let target_position = player_t.translation + rotated_offset;
    //         for (pos, opt_entity) in
    //             tree.within_distance(player_t.translation, vacuum_settings.suck_distance)
    //         {
    //             if let Some(entity) = opt_entity {
    //                 if let Ok((item_t, mut item_impulse, item_v)) = item_q.get_mut(entity) {
    //                     let direction = (target_position - item_t.translation).normalize_or_zero();
    //                     let impulse = direction * vacuum_settings.suck_to_force;
    //                     item_impulse.impulse += impulse;
    //                 }
    //             }
    //         }
    //     }
    // }

    if mouse_input.pressed(MouseButton::Left) {
        if let Ok(player_t) = player_q.get_single_mut() {
            let forward_offset = Vec3::new(0.0, 1.0, -1.3);
            let rotated_offset = player_t.rotation * forward_offset;
            let target_position = player_t.translation + rotated_offset;
            let window = window_q.single();
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok((camera, cam_transform)) = camera_q.get_single() {
                    if let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) {
                        let rapier_context = rapier_context_q.single();
                        if let Some((_, toi)) = rapier_context.cast_ray(
                            ray.origin,
                            *ray.direction,
                            f32::MAX,
                            false,
                            QueryFilter::default(),
                        ) {
                            let hit_point =
                                ray.origin + ray.direction * toi.min(vacuum_settings.suck_distance);
                            let suck_area = Collider::cuboid(1.0, 0.25, 0.25);
                            let to_player = (player_t.translation - ray.origin).normalize_or_zero();
                            spawn_vacuum_particles(
                                &mut commands,
                                &particle_assets,
                                hit_point,
                                target_position,
                            );
                            let to_player_flat =
                                Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
                            let perpendicular_rotation =
                                Quat::from_rotation_arc(Vec3::Z, to_player_flat);
                            rapier_context.intersections_with_shape(
                                hit_point,
                                perpendicular_rotation,
                                &suck_area,
                                QueryFilter::default(),
                                |nearby_entity| {
                                    let root = get_root_parent_entity(nearby_entity, &parent_query);
                                    if let Ok((item_t, mut item_impulse, item_v)) =
                                        item_q.get_mut(root)
                                    {
                                        let direction = (target_position - item_t.translation)
                                            .normalize_or_zero();
                                        let impulse = direction * vacuum_settings.suck_to_force;
                                        item_impulse.impulse += impulse;
                                    }
                                    true // continue
                                },
                            );
                            // commands.spawn((
                            //     suck_area,
                            //     CollisionGroups::new(Group::NONE, Group::NONE),
                            //     Transform::from_translation(hit_point).with_rotation(perpendicular_rotation),
                            // ));
                        }
                    }
                }
            }
        }
    }
}

fn spawn_vacuum_particles(
    mut commands: &mut Commands,
    particle: &Res<VacuumParticleAssets>,
    from: Vec3,
    to: Vec3,
) {
    let mut rng = rng();
    let dir_to = (to - from).normalize_or_zero();
    for _ in 0..10 {
        let offset_distance = rng.random_range(0.5..1.5);
        let offset_position = from + dir_to * offset_distance;
        let noise = Vec3::new(
            rng.random_range(0.0..2.0),
            rng.random_range(0.0..0.5),
            rng.random_range(0.0..0.5),
        );
        let t = Transform::from_translation(offset_position + noise).looking_at(to, Dir3::Y)
            * Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
        commands.queue(spawn_particle_t(
            particle.mesh.clone(),
            particle.material.clone(),
            t,
            rng.random_range(0.5..1.5),
            dir_to * rng.random_range(2.0..4.0),
        ));
    }
}
