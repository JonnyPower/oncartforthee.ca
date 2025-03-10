use crate::state::InGameState;
use bevy::app::{App, Update};
use bevy::asset::{Assets, Handle};
use bevy::color::palettes::basic::WHITE;
use bevy::math::Vec3;
use bevy::pbr::{Material, MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    in_state, Command, Commands, Component, Entity, FloatExt, FromWorld, IntoSystemConfigs, Mesh,
    Mesh3d, Plugin, Query, Res, Resource, Sphere, StableInterpolate, Time, Timer, TimerMode,
    Transform, World,
};

pub struct ParticlesPlugin;
impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (simulate_particles).run_if(in_state(InGameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Particle {
    lifeteime_timer: Timer,
    size: f32,
    velocity: Vec3,
}

pub fn spawn_particle<M: Material>(
    mesh: Handle<Mesh>,
    material: Handle<M>,
    translation: Vec3,
    lifetime: f32,
    size: f32,
    velocity: Vec3,
) -> impl Command {
    spawn_particle_t(
        mesh,
        material,
        Transform {
            translation,
            scale: Vec3::splat(size),
            ..Default::default()
        },
        lifetime,
        velocity,
    )
}

pub fn spawn_particle_t<M: Material>(
    mesh: Handle<Mesh>,
    material: Handle<M>,
    t: Transform,
    lifetime: f32,
    velocity: Vec3,
) -> impl Command {
    move |world: &mut World| {
        world.spawn((
            Particle {
                lifeteime_timer: Timer::from_seconds(lifetime, TimerMode::Once),
                size: t.scale.x,
                velocity,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            t,
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
