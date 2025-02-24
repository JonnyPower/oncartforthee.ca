use crate::game::effects::particles::{spawn_particle, ParticleAssets};
use crate::game::game::Player;
use crate::state::InGameState;
use bevy::animation::{AnimationPlayer, AnimationTarget};
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::hierarchy::Children;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{
    debug, in_state, warn, Added, AnimationClip, AnimationGraph, AnimationGraphHandle,
    AnimationNodeIndex, AnimationNodeType, AnimationTransitions, Assets, Commands, Component,
    Entity, Event, HierarchyQueryExt, IntoSystemConfigs, Parent, Query, Reflect, Res, ResMut,
    Transform, Trigger, With,
};
use bevy::scene::SceneInstanceReady;
use rand::{rng, Rng};
use std::ops::Div;
use std::time::Duration;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (link_animations).run_if(in_state(InGameState::Playing)),
        );
        app.add_observer(observe_on_step);
    }
}

#[derive(Component)]
pub struct AnimationPlayerEntityForRootEntity(pub Entity);

#[derive(Component)]
pub struct AnimationToPlay {
    pub graph_handle: Handle<AnimationGraph>,
    pub index: AnimationNodeIndex,
}

#[derive(Event, Reflect, Clone)]
pub struct PlayerOnStep;

fn link_animations(
    animation_player_q: Query<Entity, Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    existing_link_q: Query<&AnimationPlayerEntityForRootEntity>,
    mut commands: Commands,
) {
    // Find entites where AnimationPlayer is added
    for entity in animation_player_q.iter() {
        let root = get_root_parent_entity(entity, &parent_query);
        if existing_link_q.get(entity).is_ok() {
            warn!("Additional animation player added for root entity!");
        } else {
            commands
                .entity(root)
                .insert(AnimationPlayerEntityForRootEntity(entity.clone()));
            debug!("Added link to animation player entity");
        }
    }
}

pub fn setup_animation_graph(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
    mut clips: ResMut<Assets<AnimationClip>>,
    graphs: Res<Assets<AnimationGraph>>,
) {
    // Have to find the existing clip in the graph to add events to it
    fn get_clip<'a>(
        node: AnimationNodeIndex,
        graph: &AnimationGraph,
        clips: &'a mut Assets<AnimationClip>,
    ) -> &'a mut AnimationClip {
        let node = graph.get(node).unwrap();
        let clip = match &node.node_type {
            AnimationNodeType::Clip(handle) => clips.get_mut(handle),
            _ => unreachable!(),
        };
        clip.unwrap()
    }

    if let Ok(animation_to_play) = animations_to_play.get(trigger.entity()) {
        for child in children.iter_descendants(trigger.entity()) {
            if let Ok(mut player) = players.get_mut(child) {
                info!("found animation player, adding graph handle");
                let graph = graphs.get(&animation_to_play.graph_handle).unwrap();
                let animation_clip = get_clip(animation_to_play.index, graph, &mut clips);
                animation_clip.add_event(0.25, PlayerOnStep);
                animation_clip.add_event(0.58, PlayerOnStep);
                info!("clips added");
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, animation_to_play.index, Duration::ZERO)
                    .repeat();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()))
                    .insert(transitions);
            }
        }
    } else {
        info!("animation player not found");
    }
}

fn get_root_parent_entity(mut curr_entity: Entity, parent_query: &Query<&Parent>) -> Entity {
    //Loop up all the way to the top parent
    loop {
        if let Ok(parent) = parent_query.get(curr_entity) {
            curr_entity = parent.get();
        } else {
            break;
        }
    }
    curr_entity
}

fn observe_on_step(
    trigger: Trigger<PlayerOnStep>,
    mut commands: Commands,
    transform_q: Query<&Transform, With<Player>>,
    particle: Res<ParticleAssets>,
) {
    if let Ok(player_t) = transform_q.get_single() {
        let mut rng = rng();
        // Spawn a bunch of particles.
        for _ in 0..12 {
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
}
