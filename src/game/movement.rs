use std::ops::Div;
use bevy::app::App;
use bevy::color::palettes::basic::WHITE;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{debug, in_state, info, Assets, ButtonInput, Camera, Command, Commands, Component, Dir2, Entity, FixedUpdate, FloatExt, FromWorld, Handle, IntoSystemConfigs, KeyCode, Material, Mesh, Mesh3d, MeshMaterial3d, Plugin, Query, Res, Resource, Sphere, StableInterpolate, StandardMaterial, Timer, TimerMode, Transform, Update, Vec3Swizzles, With, Without, World};
use bevy::time::Time;
use bevy_rapier3d::prelude::Velocity;
use rand::{rng, Rng};
use crate::camera::GameCamera;
use crate::game::game::Player;
use crate::state::InGameState;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (handle_movement).run_if(in_state(InGameState::Playing)));
        app.add_systems(Update, (simulate_particles).run_if(in_state(InGameState::Playing)));
        app.init_resource::<ParticleAssets>();
    }
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);
impl Default for MovementSpeed {
    fn default() -> Self {
        MovementSpeed(1.0)
    }
}


#[derive(Resource)]
struct ParticleAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
impl FromWorld for ParticleAssets {
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

#[derive(Component)]
struct Particle {
    lifeteime_timer: Timer,
    size: f32,
    velocity: Vec3,
}

fn handle_movement(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut Transform, &mut Velocity, &MovementSpeed), With<Player>>,
    mut camera_q: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
    particle: Res<ParticleAssets>,
    time: Res<Time>
) {
    let mut xz_plane_movement = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        xz_plane_movement += Vec2::new(-1.0, 0.0);
    }

    if keys.pressed(KeyCode::KeyS) {
        xz_plane_movement += Vec2::new(1.0, 0.0);
    }

    if keys.pressed(KeyCode::KeyA) {
        xz_plane_movement += Vec2::new(0.0, 1.0);
    }

    if keys.pressed(KeyCode::KeyD) {
        xz_plane_movement += Vec2::new(0.0, -1.0);
    }

    match player_q.get_single_mut() {
        Ok((mut player_t, mut player_velocity, speed)) => {
            if xz_plane_movement != Vec2::ZERO {
                let change = xz_plane_movement.normalize_or_zero() * speed.0 * time.delta_secs();
                let new_location = player_t.translation + Vec3::new(change.x, 0.0, change.y);
                player_t.look_at(new_location, Vec3::Y);
                player_velocity.linvel = (player_velocity.linvel + Vec3::new(change.x, 0.0, change.y)).clamp(Vec3::splat(-250.0), Vec3::splat(250.0));
                let mut rng = rng();
                // Spawn a bunch of particles.
                for _ in 0..3 {
                    let size = rng.random_range(0.01..0.03);
                    let particle_spawn = player_t.translation + player_t.back().div(Vec3::splat(2.5)) + Vec3::new(rng.random_range(-0.25..0.25), 0., rng.random_range(-0.25..0.25));
                    commands.queue(spawn_particle(
                        particle.mesh.clone(),
                        particle.material.clone(),
                        particle_spawn.reject_from_normalized(Vec3::Y),
                        rng.random_range(0.05..0.15),
                        size,
                        Vec3::new(player_t.back().x + rng.random_range(-0.5..0.5), rng.random_range(0.0..4.0), player_t.back().z + rng.random_range(-0.5..0.5)),
                    ));
                }
            }
            let mut camera_t = camera_q.single_mut();
            camera_t.translation = camera_t.translation.lerp(
                Vec3::new(player_t.translation.x + 2.0, camera_t.translation.y, player_t.translation.z),
                time.delta_secs() * 5.0
            );
        }
        _ => {}
    }
    
}

fn spawn_particle<M: Material>(
    mesh: Handle<Mesh>,
    material: Handle<M>,
    translation: Vec3,
    lifetime: f32,
    size: f32,
    velocity: Vec3,
) -> impl Command {
    move |world: &mut World| {
        world.spawn((
            Particle {
                lifeteime_timer: Timer::from_seconds(lifetime, TimerMode::Once),
                size,
                velocity,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform {
                translation,
                scale: Vec3::splat(size),
                ..Default::default()
            },
        ));
    }
}

fn simulate_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in &mut query {
        if particle.lifeteime_timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * time.delta_secs();
            transform.scale =
                Vec3::splat(particle.size.lerp(0.0, particle.lifeteime_timer.fraction()));
            particle
                .velocity
                .smooth_nudge(&Vec3::ZERO, 4.0, time.delta_secs());
        }
    }
}