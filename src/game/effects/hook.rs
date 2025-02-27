use crate::game::item::ItemPickup;
use crate::game::player::Player;
use crate::state::InGameState;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{
    in_state, Commands, Component, Entity, IntoSystemConfigs, MouseButton, Plugin, Query, Reflect,
    Res, Resource, Time, Transform, Update, With,
};
use bevy::prelude::{
    Click, Down, Pointer, ReflectResource, Timer, TimerMode, Trigger, Up, Without,
};
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;
use blenvy::Observer;
use rand::Rng;

pub struct PlayerSkillHookPlugin;
impl Plugin for PlayerSkillHookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_hooked_items, shake_effect_system).run_if(in_state(InGameState::Playing)),
        );
        app.insert_resource(HookResource {
            hook_range: 5.0,
            hooked_item_speed: 3.0,
        });
        app.register_type::<HookResource>();
        app.add_plugins(ResourceInspectorPlugin::<HookResource>::default());
    }
}

#[derive(Component)]
pub struct ItemIsHooked;

#[derive(Resource, InspectorOptions, Reflect)]
#[reflect(Resource, InspectorOptions)]
pub struct HookResource {
    hook_range: f32,
    hooked_item_speed: f32,
}

#[derive(Component)]
pub struct ShakeEffect {
    duration: f32,
    timer: Timer,
    original_position: Vec3,
}
impl ShakeEffect {
    pub fn new(duration: f32, original_position: Vec3) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            original_position,
        }
    }
}

// FIXME contain to this module by adding observer on newly spawned item pickups, or figure out how global observers work
pub fn hook_item_on_click(
    trigger: Trigger<Pointer<Down>>,
    mut commands: Commands,
    q_picked: Query<(Entity, &Transform), With<ItemPickup>>,
    player_query: Query<&Transform, (With<Player>, Without<ItemIsHooked>)>,
    hook_settings: Res<HookResource>,
) {
    if let Ok((entity, item_t)) = q_picked.get(trigger.entity()) {
        if let Ok(player_t) = player_query.get_single() {
            if item_t
                .translation
                .distance_squared(player_t.translation)
                .abs()
                < hook_settings.hook_range.powi(2)
            {
                commands.entity(entity).insert(ItemIsHooked);
            } else {
                commands
                    .entity(entity)
                    .insert(ShakeEffect::new(0.3, item_t.translation));
            }
        }
    }
}

pub fn shake_effect_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ShakeEffect)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut transform, mut shake) in query.iter_mut() {
        if shake.timer.tick(time.delta()).finished() {
            // Reset position and remove the effect after the timer ends
            transform.translation = shake.original_position;
            commands.entity(entity).remove::<ShakeEffect>();
        } else {
            // Apply random shake effect
            let offset_x = rng.random_range(-0.02..0.02);
            let offset_y = rng.random_range(-0.02..0.02);
            transform.translation = shake.original_position + Vec3::new(offset_x, offset_y, 0.0);
        }
    }
}

fn move_hooked_items(
    time: Res<Time>,
    mut query: Query<(&mut Transform, Entity), With<ItemIsHooked>>,
    player_query: Query<&Transform, (With<Player>, Without<ItemIsHooked>)>,
    hook_settings: Res<HookResource>,
) {
    if let Ok(player_t) = player_query.get_single() {
        let forward_offset = Vec3::new(0.0, 0.5, -1.3);
        let rotated_offset = player_t.rotation * forward_offset;
        let target_position = player_t.translation + rotated_offset;

        for (mut transform, entity) in query.iter_mut() {
            let progress = time.delta_secs() * hook_settings.hooked_item_speed;
            let midpoint = (transform.translation + target_position) / 2.0 + Vec3::Y * 2.0;
            let new_position = transform.translation.lerp(midpoint, progress);
            transform.translation = new_position.lerp(target_position, progress);
        }
    }
}
