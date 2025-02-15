use bevy::app::App;
use bevy::prelude::{AppExtStates, NextState, OnEnter, OnExit, Plugin, ResMut, States};
use web_sys::console::debug;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    TitleMenu,
    InGame
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum TitleMenuState {
    Hidden,
    #[default]
    Home,
    NewGame,
    Settings
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum InGameState {
    #[default]
    None,
    Playing
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(AppState::TitleMenu),
            hide_title_on_state
        );
        app.add_systems(
            OnEnter(AppState::TitleMenu),
            show_title_on_state
        );
        app.add_systems(
            OnExit(AppState::InGame),
            stop_game_on_state
        );
        app.add_systems(
            OnEnter(AppState::InGame),
            initial_in_game_state
        );
        app.init_state::<AppState>();
        app.init_state::<TitleMenuState>();
        app.init_state::<InGameState>();
    }
}


fn hide_title_on_state(mut title_menu_state: ResMut<NextState<TitleMenuState>>) {
    title_menu_state.set(TitleMenuState::Hidden);
}

fn show_title_on_state(mut title_menu_state: ResMut<NextState<TitleMenuState>>) {
    title_menu_state.set(TitleMenuState::Home);
}

fn stop_game_on_state(
    mut in_game_state: ResMut<NextState<InGameState>>
) {
    in_game_state.set(InGameState::None);
}

fn initial_in_game_state(mut in_game_state: ResMut<NextState<InGameState>>) {
    in_game_state.set(InGameState::Playing);
}