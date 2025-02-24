use crate::game::game::{Player, TrackedByKDTree};
use crate::game::item::ItemPickup;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::ReflectResource;
use bevy::prelude::{
    MouseButton, Plugin, Query, Reflect, Res, Resource, Transform, Update, With, Without,
};
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::dynamics::{ExternalImpulse, Velocity};
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::SpatialAccess;

pub struct PlayerSkillVacuumPlugin;
impl Plugin for PlayerSkillVacuumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_click);
        app.insert_resource(VacuumResource {
            suck_distance: 5.0,
            suck_to_force: 0.0005,
        });
        app.register_type::<VacuumResource>();
        app.add_plugins(ResourceInspectorPlugin::<VacuumResource>::default());
    }
}

#[derive(Resource, InspectorOptions, Reflect)]
#[reflect(Resource, InspectorOptions)]
pub struct VacuumResource {
    suck_distance: f32,
    suck_to_force: f32,
}

fn handle_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    tree: Res<KDTree3<TrackedByKDTree>>,
    mut player_q: Query<&Transform, With<Player>>,
    mut item_q: Query<
        (&Transform, &mut ExternalImpulse, &Velocity),
        (Without<Player>, With<ItemPickup>),
    >,
    vacuum_settings: Res<VacuumResource>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok(player_t) = player_q.get_single_mut() {
            let forward_offset = Vec3::new(0.0, 0.75, -1.3);
            let rotated_offset = player_t.rotation * forward_offset;
            let target_position = player_t.translation + rotated_offset;
            for (pos, opt_entity) in
                tree.within_distance(player_t.translation, vacuum_settings.suck_distance)
            {
                if let Some(entity) = opt_entity {
                    if let Ok((item_t, mut item_impulse, item_v)) = item_q.get_mut(entity) {
                        let direction = (target_position - item_t.translation).normalize_or_zero();
                        let impulse = direction * vacuum_settings.suck_to_force;
                        item_impulse.impulse += impulse;
                    }
                }
            }
        }
    }
}
