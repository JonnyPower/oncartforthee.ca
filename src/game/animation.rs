use crate::state::InGameState;
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::hierarchy::Children;
use bevy::log::info;
use bevy::prelude::{
    debug, in_state, warn, Added, AnimationGraph, AnimationGraphHandle, AnimationNodeIndex,
    Commands, Component, Entity, HierarchyQueryExt, IntoSystemConfigs, Parent, Query, Trigger,
};
use bevy::scene::SceneInstanceReady;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (link_animations).run_if(in_state(InGameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct AnimationPlayerEntityForRootEntity(pub Entity);

#[derive(Component)]
pub struct AnimationToPlay {
    pub graph_handle: Handle<AnimationGraph>,
    pub index: AnimationNodeIndex,
}

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
) {
    if let Ok(animation_to_play) = animations_to_play.get(trigger.entity()) {
        for child in children.iter_descendants(trigger.entity()) {
            if let Ok(mut player) = players.get_mut(child) {
                info!("found animation player, adding graph handle");
                player.pause_all();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
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
