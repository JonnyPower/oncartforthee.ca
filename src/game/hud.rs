use crate::game::game::ScoreResource;
use crate::state::InGameState;
use bevy::app::App;
use bevy::color::Color;
use bevy::prelude::{
    in_state, AssetServer, BackgroundColor, BuildChildren, ChildBuild, Commands, Component, Entity,
    IntoSystemConfigs, LinearRgba, Node, OnEnter, Parent, Plugin, PositionType, Query, Res, Text,
    Update, Val, With, Without,
};
use bevy::text::TextSpan;
use bevy_egui::egui::MouseWheelUnit::Line;
use rand::Rng;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), setup_hud);
        app.add_systems(Update, (update_hud).run_if(in_state(InGameState::Playing)));
    }
}

#[derive(Component)]
pub struct HudScoreText;

#[derive(Component)]
struct SendItMeter;

#[derive(Component)]
struct SendItText;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Text::new("Score: "), HudScoreText))
        .with_child((TextSpan::default(), HudScoreText));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(30.0),
                height: Val::Px(30.0),
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..Default::default()
            },
            BackgroundColor(Color::LinearRgba(LinearRgba::new(0.5, 0.5, 0.5, 1.0))),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    ..Default::default()
                },
                BackgroundColor(Color::LinearRgba(LinearRgba::new(0.2, 0.2, 0.2, 1.0))),
                SendItMeter,
            ));
        });
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(30.0),
            height: Val::Px(30.0),
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..Default::default()
        },
        Text::new("SEND IT!"),
        SendItText,
    ));
    commands.spawn((
        Text::new("WASD - Move\nSpace - Stomp\nShift - Run\nRight Click + Scroll - Camera"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..Default::default()
        },
    ));
}

fn update_hud(
    score_res: Res<ScoreResource>,
    mut score_text_q: Query<&mut TextSpan, With<HudScoreText>>,
    mut send_it_meter_q: Query<(&Parent, &mut Node, &mut BackgroundColor), With<SendItMeter>>,
    mut node_q: Query<(&mut Node), (Without<SendItMeter>, Without<SendItText>)>,
    mut send_it_text: Query<&mut Node, (Without<SendItMeter>, With<SendItText>)>,
) {
    for mut span in &mut score_text_q {
        let score = score_res.score;
        **span = format!("{score:.2}");
    }
    let send_it_progress = score_res.score.clamp(0, 100) as f32;
    let mut rng = rand::rng();
    let shake_intensity = send_it_progress / 2.0;
    let offset_x = rng.random_range(-shake_intensity..=shake_intensity);
    let offset_y = rng.random_range(-shake_intensity..=shake_intensity);
    if let Ok((parent_e, mut send_it_node, mut send_it_bg)) = send_it_meter_q.get_single_mut() {
        send_it_node.width = Val::Percent(send_it_progress);
        let color = Color::LinearRgba(LinearRgba::new(
            send_it_progress,
            1.0 - send_it_progress,
            0.0,
            1.0,
        ));
        *send_it_bg = BackgroundColor(color);
        if let Ok((mut parent_node)) = node_q.get_mut(**parent_e) {
            if let Ok(mut send_it_text_node) = send_it_text.get_single_mut() {
                parent_node.left = Val::Px(20.0 + offset_x);
                parent_node.bottom = Val::Px(20.0 + offset_y);
                send_it_text_node.left = Val::Px(20.0 + offset_x);
                send_it_text_node.bottom = Val::Px(20.0 + offset_y);
            }
        }
    }
}
