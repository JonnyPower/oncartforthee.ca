use crate::state::{AppState, TitleMenuState};
use bevy::app::App;
use bevy::prelude::{
    default, in_state, AssetServer, Commands, Component, DespawnRecursiveExt, Display, Entity,
    ImageNode, IntoSystemConfigs, NextState, Node, OnEnter, OnExit, Plugin, PositionType, Query,
    Res, ResMut, Update, Val, With,
};
use bevy_egui::egui::{Button, Color32, Frame, Response, RichText, TextStyle, Ui, Vec2};
use bevy_egui::{egui, EguiContexts};

pub struct UITitleMenuHomePlugin;
impl Plugin for UITitleMenuHomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TitleMenuState::Home), title_menu_setup);
        app.add_systems(OnExit(TitleMenuState::Home), title_menu_cleanup);
        app.add_systems(
            Update,
            title_menu_system.run_if(in_state(TitleMenuState::Home)),
        );
    }
}

const PANEL_WIDTH: f32 = 300.0;
const PANEL_BUTTON_SIZE: Vec2 = Vec2::new(286.0, 40.0);

#[derive(Component)]
struct TitleMenuTag;

fn title_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title_back = asset_server.load("images/title_back.png");
    commands.spawn((
        ImageNode::new(title_back),
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..default()
        },
        TitleMenuTag,
    ));
}

fn title_menu_system(
    mut contexts: EguiContexts,
    mut title_menu_state: ResMut<NextState<TitleMenuState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    egui::SidePanel::left("title_left_panel")
        .frame(
            Frame::default()
                .inner_margin(8.)
                .fill(Color32::from_black_alpha(200)),
        )
        .resizable(false)
        .show_separator_line(false)
        .exact_width(PANEL_WIDTH)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(
                RichText::new("On Cart for Thee")
                    .text_style(TextStyle::Heading)
                    .size(32.),
            );
            new_game_button(ui, &mut app_state);
            settings_button(ui, &mut title_menu_state);
        });
}

fn title_menu_cleanup(cleanup: Query<Entity, With<TitleMenuTag>>, mut commands: Commands) {
    for entity in &cleanup {
        commands.entity(entity).despawn_recursive();
    }
}

fn title_button(ui: &mut Ui, text: &str) -> Response {
    ui.add_sized(
        PANEL_BUTTON_SIZE,
        Button::new(RichText::new(text).size(22.)),
    )
}

fn new_game_button(ui: &mut Ui, app_state: &mut ResMut<NextState<AppState>>) {
    if title_button(ui, "New Game").clicked() {
        app_state.set(AppState::InGame);
    }
}

fn settings_button(ui: &mut Ui, title_menu_state: &mut ResMut<NextState<TitleMenuState>>) {
    if title_button(ui, "Settings").clicked() {
        title_menu_state.set(TitleMenuState::Settings)
    }
}
