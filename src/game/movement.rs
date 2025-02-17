use crate::camera::GameCamera;
use crate::game::game::{AnimationToPlay, Player};
use crate::game::particles::{spawn_particle, ParticleAssets};
use crate::state::InGameState;
use bevy::animation::RepeatAnimation;
use bevy::app::App;
use bevy::color::palettes::basic::WHITE;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    debug, in_state, info, warn, AnimationPlayer, Assets, ButtonInput, Camera, Command, Commands,
    Component, Dir2, Entity, FixedUpdate, FloatExt, FromWorld, Handle, IntoSystemConfigs, KeyCode,
    Material, Mesh, Mesh3d, MeshMaterial3d, Plugin, Quat, Query, Reflect, Res, Resource, Sphere,
    StableInterpolate, StandardMaterial, Timer, TimerMode, Transform, Update, Vec3Swizzles, With,
    Without, World,
};
use bevy::time::Time;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::{ExternalImpulse, Velocity};
use rand::{rng, Rng};
use std::ops::Div;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_movement).run_if(in_state(InGameState::Playing)),
        );
        app.register_type::<MovementSettings>();
    }
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementSettings {
    pub speed: f32,
    pub max_speed: f32,
}
impl Default for MovementSettings {
    fn default() -> Self {
        MovementSettings {
            speed: 10.0,
            max_speed: 100.0,
        }
    }
}

fn handle_movement(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<
        (
            Entity,
            &mut Transform,
            &Velocity,
            &mut ExternalImpulse,
            &MovementSettings,
            &AnimationToPlay,
        ),
        With<Player>,
    >,
    mut animationp_q: Query<(&mut AnimationPlayer), Without<Player>>,
    mut camera_q: Query<
        &mut Transform,
        (With<GameCamera>, Without<Player>, Without<AnimationPlayer>),
    >,
    particle: Res<ParticleAssets>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction -= Vec3::X;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction += Vec3::X;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction += Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction -= Vec3::Z;
    }

    match player_q.get_single_mut() {
        Ok((
            player_e,
            mut player_t,
            player_velocity,
            mut player_impulse,
            player_ms,
            player_animation,
        )) => {
            if direction != Vec3::ZERO {
                direction = direction.normalize();
                let impulse_force = direction
                    * player_ms.speed
                    * if keys.pressed(KeyCode::ShiftLeft) {
                        2.0
                    } else {
                        1.0
                    };
                if player_velocity.linvel.length() < player_ms.max_speed {
                    player_impulse.impulse += impulse_force;
                    draw_run_particles(&mut commands, &player_t, &particle);
                }
                if player_velocity.linvel.length_squared() > 0.1 {
                    let facing_direction = player_velocity.linvel.normalize();
                    let target_rotation = Quat::from_rotation_arc(
                        Vec3::NEG_Z,
                        Vec3::new(facing_direction.x, 0.0, facing_direction.z),
                    );
                    player_t.rotation = target_rotation;
                }
            }

            if let Ok(mut animation_p) = animationp_q.get_single_mut() {
                let opt_animation = animation_p.animation_mut(player_animation.index);
                if player_velocity.linvel.length_squared() > 1.0 {
                    match opt_animation {
                        Some(animation) if animation.is_paused() => {
                            animation.resume();
                        }
                        None => {
                            animation_p.play(player_animation.index).repeat();
                        }
                        _ => {}
                    }
                } else {
                    if let Some(animation) = opt_animation {
                        animation.pause().set_seek_time(0.0);
                    }
                }
            }

            let mut camera_t = camera_q.single_mut();
            camera_t.translation = camera_t.translation.lerp(
                Vec3::new(
                    player_t.translation.x + 5.0,
                    camera_t.translation.y,
                    player_t.translation.z,
                ),
                1.0 - (-time.delta_secs() * 5.0).exp(),
            );
        }
        _ => {
            warn!("Player not found");
        }
    }
}

fn draw_run_particles(
    mut commands: &mut Commands,
    player_t: &Transform,
    particle: &Res<ParticleAssets>,
) {
    let mut rng = rng();
    // Spawn a bunch of particles.
    for _ in 0..3 {
        let size = rng.random_range(0.01..0.03);
        let particle_spawn = player_t.translation
            + player_t.back().div(Vec3::splat(2.5))
            + Vec3::new(
                rng.random_range(-0.25..0.25),
                0.,
                rng.random_range(-0.25..0.25),
            );
        commands.queue(spawn_particle(
            particle.mesh.clone(),
            particle.material.clone(),
            particle_spawn.reject_from_normalized(Vec3::Y),
            rng.random_range(0.05..0.15),
            size,
            Vec3::new(
                player_t.back().x + rng.random_range(-0.5..0.5),
                rng.random_range(0.0..4.0),
                player_t.back().z + rng.random_range(-0.5..0.5),
            ),
        ));
    }
}
