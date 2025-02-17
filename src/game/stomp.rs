use crate::game::game::{Player, TrackedByKDTree};
use crate::game::item::ItemPickup;
use crate::state::InGameState;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{
    in_state, Commands, IntoSystemConfigs, KeyCode, Plugin, Query, Res, Transform, Update, With,
    Without,
};
use bevy_rapier3d::prelude::ExternalImpulse;
use bevy_spatial::kdtree::KDTree3;
use bevy_spatial::SpatialAccess;

const STOMP_DISTANCE: f32 = 2.0;
const STOMP_IMPULSE: Vec3 = Vec3::new(0.0, 0.1, 0.0);

pub struct PlayerStompPlugin;
impl Plugin for PlayerStompPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_stomp).run_if(in_state(InGameState::Playing)),
        );
    }
}

fn handle_stomp(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&Transform, With<Player>>,
    tree: Res<KDTree3<TrackedByKDTree>>,
    mut item_q: Query<(&Transform, &mut ExternalImpulse), (Without<Player>, With<ItemPickup>)>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Ok(player_t) = player_q.get_single_mut() {
            for (pos, opt_entity) in tree.within_distance(player_t.translation, STOMP_DISTANCE) {
                if let Some(entity) = opt_entity {
                    if let Ok((item_t, mut item_impulse)) = item_q.get_mut(entity) {
                        item_impulse.impulse += STOMP_IMPULSE;
                    }
                }
            }
        }
    }
}
