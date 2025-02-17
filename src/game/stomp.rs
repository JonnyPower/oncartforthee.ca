use crate::game::game::{Player, TrackedByKDTree};
use crate::game::item::ItemPickup;
use crate::game::particles::{spawn_particle, ParticleAssets};
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
use rand::{rng, Rng};
use std::f32::consts::{PI, TAU};
use std::ops::Div;

const STOMP_DISTANCE: f32 = 3.0;
const STOMP_AWAY_FORCE: f32 = 0.01;
const STOMP_IMPULSE: Vec3 = Vec3::new(0.0, 0.1, 0.0);
const STOMP_PARTICLES: i32 = 80;

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
    particle: Res<ParticleAssets>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Ok(player_t) = player_q.get_single_mut() {
            for (pos, opt_entity) in tree.within_distance(player_t.translation, STOMP_DISTANCE) {
                if let Some(entity) = opt_entity {
                    if let Ok((item_t, mut item_impulse)) = item_q.get_mut(entity) {
                        let direction =
                            (item_t.translation - player_t.translation).normalize_or_zero();
                        item_impulse.impulse += STOMP_IMPULSE + (direction * STOMP_AWAY_FORCE);
                    }
                }
            }
            draw_stomp_particles(&mut commands, &player_t, &particle);
        }
    }
}

fn draw_stomp_particles(
    mut commands: &mut Commands,
    player_t: &Transform,
    particle: &Res<ParticleAssets>,
) {
    let mut rng = rng();
    for i in 1..STOMP_PARTICLES {
        let size = rng.random_range(0.01..0.03);
        let particle_spawn = player_t.translation + player_t.back().div(Vec3::splat(2.0));
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
