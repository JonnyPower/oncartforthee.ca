use crate::camera::GameCamera;
use crate::game::animation::{AnimationPlayerEntityForRootEntity, AnimationToPlay};
use crate::game::player::Player;
use crate::state::InGameState;
use bevy::animation::RepeatAnimation;
use bevy::app::App;
use bevy::color::palettes::basic::WHITE;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    debug, in_state, info, warn, AnimationPlayer, AnimationTransitions, Assets, ButtonInput,
    Camera, Children, Command, Commands, Component, Dir2, Entity, FixedUpdate, FloatExt, FromWorld,
    Handle, IntoSystemConfigs, KeyCode, Material, Mesh, Mesh3d, MeshMaterial3d, Plugin, Quat,
    Query, Reflect, Res, Resource, Sphere, StableInterpolate, StandardMaterial, Timer, TimerMode,
    Transform, Update, Vec3Swizzles, With, Without, World,
};
use bevy::time::Time;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::prelude::{ExternalForce, ExternalImpulse, Velocity};
use rand::{rng, Rng};
use std::ops::{Deref, Div};
use std::time::Duration;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_movement, apply_stubborn_force).run_if(in_state(InGameState::Playing)),
        );
        app.register_type::<MovementSettings>();
        app.init_resource::<StepParticleAssets>();
    }
}

#[derive(Resource)]
pub struct StepParticleAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}
impl FromWorld for StepParticleAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world.resource_mut::<Assets<Mesh>>().add(Sphere::new(10.0)),
            material: world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: WHITE.into(),
                    ..Default::default()
                }),
        }
    }
}

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct MovementSettings {
    pub speed: f32,
    pub max_speed: f32,
    pub turn_speed_exp: f32,
    pub low_speed_turn_factor: f32,
    pub high_speed_turn_factor: f32,
    pub stubborn_cart_strength: f32,
}
impl Default for MovementSettings {
    fn default() -> Self {
        MovementSettings {
            speed: 2.0,
            max_speed: 25.0,
            turn_speed_exp: 2.0,
            low_speed_turn_factor: 0.3,
            high_speed_turn_factor: 1.2,
            stubborn_cart_strength: -0.5,
        }
    }
}

fn apply_stubborn_force(
    mut query: Query<(&Transform, &Velocity, &mut ExternalForce, &MovementSettings), With<Player>>,
) {
    // FIXME
    // for (transform, velocity, mut external_force, player_ms) in query.iter_mut() {
    //     if velocity.linvel.length() > 1.0 {
    //         // Apply force in that direction
    //         external_force.torque += Vec3::Y * player_ms.stubborn_cart_strength;
    //     }
    // }
}

fn handle_movement(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut ExternalImpulse,
            &MovementSettings,
            &Children,
            &AnimationPlayerEntityForRootEntity,
        ),
        With<Player>,
    >,
    player_animation_to_play_q: Query<(&AnimationToPlay), Without<Player>>,
    mut animationp_q: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    particle: Res<StepParticleAssets>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction -= Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    match player_q.get_single_mut() {
        Ok((
            player_e,
            mut player_t,
            mut player_velocity,
            mut player_impulse,
            player_ms,
            player_children,
            animation_player_link,
        )) => {
            if direction != Vec3::ZERO {
                direction = player_t.rotation * direction.normalize();
                let mut impulse_force = direction
                    * player_ms.speed
                    * if keys.pressed(KeyCode::ShiftLeft) {
                        2.0
                    } else {
                        1.0
                    };
                if player_velocity.linvel.length() < player_ms.max_speed {
                    player_impulse.impulse += impulse_force;
                }
                if player_velocity.linvel.length_squared() > 0.1
                    || player_velocity.angvel.length_squared() > 0.1
                {
                    let facing_direction = player_velocity.linvel.normalize();
                    let forward_dot = facing_direction.dot(player_t.rotation * Vec3::NEG_Z);
                    // Flip the target direction if moving backward
                    let adjusted_direction = if forward_dot < -0.1 {
                        -Vec3::new(facing_direction.x, 0.0, facing_direction.z)
                    } else {
                        Vec3::new(facing_direction.x, 0.0, facing_direction.z)
                    };
                    let target_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, adjusted_direction);
                    player_t.rotation = player_t
                        .rotation
                        .lerp(target_rotation, 1.0 - (-time.delta_secs() * 5.0).exp());
                }
            }

            // Find animation to play in player entity, then find animation player in entity tree to play animation
            if let Some(player_animation) = player_children
                .iter()
                .find_map(|&child| player_animation_to_play_q.get(child).ok())
            {
                if let Ok((mut animation_p, mut transitions)) =
                    animationp_q.get_mut(animation_player_link.0)
                {
                    let opt_animation = animation_p.animation_mut(player_animation.index);
                    if player_velocity.linvel.length_squared() > 1.0 {
                        match opt_animation {
                            Some(animation) if animation.is_paused() => {
                                animation.resume();
                            }
                            None => {
                                transitions
                                    .play(
                                        &mut animation_p,
                                        player_animation.index,
                                        Duration::from_millis(250),
                                    )
                                    .repeat();
                            }
                            _ => {}
                        }
                    } else {
                        if let Some(animation) = opt_animation {
                            animation.pause().set_seek_time(0.0);
                        }
                    }
                } else {
                    warn!("can't find animation player from link");
                }
            } else {
                warn!("can't find animation to play under player entity");
            }
        }
        _ => {}
    }
}
