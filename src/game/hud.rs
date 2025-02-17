use crate::game::stomp::ScoreResource;
use crate::state::InGameState;
use bevy::app::App;
use bevy::prelude::{
    in_state, AssetServer, BuildChildren, Commands, Component, IntoSystemConfigs, OnEnter, Plugin,
    Query, Res, Text, Update, With,
};
use bevy::text::TextSpan;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Playing), setup_hud);
        app.add_systems(Update, (update_hud).run_if(in_state(InGameState::Playing)));
    }
}

#[derive(Component)]
pub struct HudScoreText;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Text::new("Score: "), HudScoreText))
        .with_child((TextSpan::default(), HudScoreText));
}

fn update_hud(
    score_res: Res<ScoreResource>,
    mut score_text_q: Query<&mut TextSpan, With<HudScoreText>>,
) {
    for mut span in &mut score_text_q {
        let score = score_res.score;
        **span = format!("{score:.2}");
    }
}
