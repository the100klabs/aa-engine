use aa_ability::AttributeSet;
use aa_gameplay::{PlayerState, TrainingDummy};
use bevy::prelude::*;

#[derive(Component)]
struct HudRoot;

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                HudText { label: HudLabel::Player },
                Text::new("Player HP: --"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.9, 1.0)),
            ));
            parent.spawn((
                HudText { label: HudLabel::Dummy },
                Text::new("Dummy HP: --"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.7, 0.6)),
            ));
            parent.spawn((
                Text::new("WASD move | Mouse aim | Click/Space fire | Stay clear of dummy!"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
pub(crate) struct HudText {
    label: HudLabel,
}

#[derive(Clone, Copy)]
pub(crate) enum HudLabel {
    Player,
    Dummy,
}

pub fn update_hud(
    players: Query<&AttributeSet, With<PlayerState>>,
    dummies: Query<&AttributeSet, With<TrainingDummy>>,
    mut texts: Query<(&HudText, &mut Text)>,
) {
    let player_hp = players
        .iter()
        .next()
        .and_then(|a| a.get("Health"))
        .unwrap_or(0.0);
    let dummy_hp = dummies
        .iter()
        .next()
        .and_then(|a| a.get("Health"))
        .unwrap_or(0.0);

    for (hud, mut text) in &mut texts {
        match hud.label {
            HudLabel::Player => *text = Text::new(format!("Player HP: {player_hp:.0}")),
            HudLabel::Dummy => *text = Text::new(format!("Dummy HP: {dummy_hp:.0}")),
        }
    }
}
