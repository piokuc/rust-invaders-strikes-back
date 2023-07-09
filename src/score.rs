
use bevy::prelude::*;

pub(crate) const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub(crate) const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
pub(crate) const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub(crate) const TEXT_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

// This resource tracks the game's score
#[derive(Resource)]
pub(crate) struct Scoreboard {
    pub(crate) score: usize,
}
#[derive(Component)]
pub(crate) struct ScoreText;


pub(crate) fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

pub(crate) fn setup_scoreboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>) {
    // Scoreboard
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 60.0, color: Color::WHITE, },
            ),
            TextSection::from_style(TextStyle { font: asset_server.load("fonts/FiraMono-Medium.ttf"), font_size: 60.0, color: Color::GOLD, }),
        ]),
        ScoreText,
    ));
}