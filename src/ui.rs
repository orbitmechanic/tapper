//! HUD, title, pause and game-over presentation.

use bevy::prelude::*;

use crate::AppState;
use crate::game::Game;
use crate::input::PadInput;
use crate::render::Art;
use crate::world::{THEME_NAMES, THEME_SUBS};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudField {
    Score,
    Screen,
    Theme,
    Banner,
    Hint,
}

#[derive(Component)]
pub struct HudTop;

#[derive(Component)]
pub struct LifeIcon(pub u32);

#[derive(Component)]
pub struct PauseVeil;

#[derive(Component)]
pub struct TitleRoot;

#[derive(Component)]
pub struct TitlePrompt;

#[derive(Component)]
pub struct TitleHigh;

#[derive(Component)]
pub struct GameOverScore;

fn fonted(art: &Art, content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content.into()),
        TextFont {
            font: art.font.clone().into(),
            font_size: FontSize::Px(size),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::new(2.0, 2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
        },
    )
}

fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(gap),
        ..default()
    }
}

pub fn spawn_hud(commands: &mut Commands, art: &Art, touch_mode: bool) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(58.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(22.0),
                    right: Val::Px(22.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.024, 0.03)),
            ZIndex(50),
            HudTop,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn(row(10.0)).with_children(|r| {
                r.spawn(fonted(art, "SCORE", 17.0, Color::srgb(0.95, 0.66, 0.25)));
                r.spawn((
                    fonted(art, "000000", 31.0, Color::srgb(1.0, 0.95, 0.8)),
                    HudField::Score,
                ));
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    fonted(art, "SALOON", 20.0, Color::srgb(0.45, 0.9, 0.82)),
                    HudField::Theme,
                ));
                c.spawn((
                    fonted(art, "SCREEN 1/13", 16.0, Color::srgb(0.72, 0.76, 0.84)),
                    HudField::Screen,
                ));
            });

            bar.spawn(row(8.0)).with_children(|r| {
                r.spawn(fonted(art, "SHIFTS", 15.0, Color::srgb(0.95, 0.66, 0.25)));
                for i in 0..3 {
                    r.spawn((
                        ImageNode::new(art.mug_full.clone()),
                        Node {
                            width: Val::Px(21.0),
                            height: Val::Px(25.0),
                            ..default()
                        },
                        LifeIcon(i),
                    ));
                }
            });
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(58.0),
            width: Val::Percent(100.0),
            height: Val::Px(3.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.9, 0.44, 0.16, 0.9)),
        DespawnOnExit(AppState::InGame),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.024, 0.03)),
            ZIndex(50),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn((
                fonted(
                    art,
                    if touch_mode {
                        "PAD: LANES / RUN   POUR: SERVE   II: PAUSE"
                    } else {
                        "W/S LANE  A/D RUN  SPACE POUR  P PAUSE  M MUTE"
                    },
                    17.0,
                    Color::srgb(0.66, 0.72, 0.8),
                ),
                HudField::Hint,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(112.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn((
                fonted(art, "", 30.0, Color::srgb(1.0, 0.85, 0.3)),
                HudField::Banner,
            ));
        });

    // Pause veil lives above the HUD and is toggled by `sync_pause`.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.014, 0.02, 0.8)),
            Visibility::Hidden,
            PauseVeil,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|veil| {
            veil.spawn(fonted(
                art,
                "LAST CALL PAUSED",
                42.0,
                Color::srgb(1.0, 0.82, 0.32),
            ));
            veil.spawn(fonted(
                art,
                "P / ESC TO RETURN TO THE BAR",
                18.0,
                Color::srgb(0.8, 0.84, 0.9),
            ));
        });
}

pub fn update_hud(
    game: Res<Game>,
    mut fields: Query<(&HudField, &mut Text)>,
    mut lives: Query<(&LifeIcon, &mut Visibility)>,
) {
    for (field, mut text) in &mut fields {
        match field {
            HudField::Score => text.0 = format!("{:06}", game.score),
            HudField::Screen => {
                text.0 = format!(
                    "SCREEN {}/{}  ·  LOOP {}",
                    game.screen + 1,
                    crate::world::SCREEN_COUNT,
                    game.loop_no + 1
                )
            }
            HudField::Theme => text.0 = THEME_NAMES[game.theme()].to_string(),
            HudField::Banner => text.0 = game.banner.clone(),
            HudField::Hint => {}
        }
    }
    for (icon, mut vis) in &mut lives {
        *vis = if icon.0 < game.lives {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn spawn_hud_system(mut commands: Commands, art: Res<Art>, touch: Res<PadInput>) {
    spawn_hud(&mut commands, &art, touch.touch_mode);
}

pub fn toggle_pause_veil(
    paused: Res<crate::Paused>,
    mut q: Query<&mut Visibility, With<PauseVeil>>,
) {
    for mut vis in &mut q {
        *vis = if paused.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn spawn_title(commands: &mut Commands, art: &Art, high: u32) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit(AppState::Title),
            TitleRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(art.background(0)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.012, 0.02, 0.55)),
            ));
            root.spawn((
                Node {
                    width: Val::Px(640.0),
                    height: Val::Px(174.0),
                    margin: UiRect {
                        bottom: Val::Px(26.0),
                        ..default()
                    },
                    ..default()
                },
                ImageNode::new(art.logo.clone()),
            ));
            root.spawn(fonted(
                art,
                "FOUR BARS. FOUR CROWDS. ONE BARTENDER.",
                20.0,
                Color::srgb(0.98, 0.83, 0.55),
            ));
            root.spawn((
                fonted(
                    art,
                    "PRESS ENTER / TAP TO START",
                    27.0,
                    Color::srgb(0.55, 0.95, 0.85),
                ),
                TitlePrompt,
            ));
            root.spawn((
                fonted(
                    art,
                    format!("HIGH SCORE  {:06}", high),
                    20.0,
                    Color::srgb(0.9, 0.9, 0.94),
                ),
                TitleHigh,
            ));
            root.spawn(Node {
                margin: UiRect {
                    top: Val::Px(26.0),
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(26.0),
                ..default()
            })
            .with_children(|row| {
                for (key, what) in [
                    ("W/S", "CHANGE LANE"),
                    ("A/D", "RUN THE BAR"),
                    ("SPACE", "POUR & SERVE"),
                    ("TIP", "DISTRACT THE CROWD"),
                ] {
                    row.spawn(fonted(
                        art,
                        format!("{key}  {what}"),
                        16.0,
                        Color::srgb(0.7, 0.75, 0.82),
                    ));
                }
            });
        });
}

pub fn update_title(
    time: Res<Time>,
    game: Res<Game>,
    mut prompt: Query<&mut TextColor, With<TitlePrompt>>,
    mut high: Query<&mut Text, With<TitleHigh>>,
) {
    let blink = (time.elapsed_secs() * 2.2).sin() > -0.15;
    for mut color in &mut prompt {
        color.0 = if blink {
            Color::srgb(0.55, 0.98, 0.86)
        } else {
            Color::srgba(0.55, 0.98, 0.86, 0.25)
        };
    }
    for mut text in &mut high {
        text.0 = format!("HIGH SCORE  {:06}", game.high_score);
    }
}

pub fn spawn_game_over(commands: &mut Commands, art: &Art, game: &Game) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.012, 0.018, 0.86)),
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|root| {
            root.spawn(fonted(
                art,
                "THE BAR IS CLOSED",
                48.0,
                Color::srgb(0.95, 0.4, 0.3),
            ));
            root.spawn((
                fonted(
                    art,
                    format!("FINAL SCORE  {:06}", game.score),
                    30.0,
                    Color::srgb(1.0, 0.9, 0.65),
                ),
                GameOverScore,
            ));
            root.spawn(fonted(
                art,
                format!(
                    "REACHED SCREEN {}  ·  LOOP {}",
                    game.screen + 1,
                    game.loop_no + 1
                ),
                20.0,
                Color::srgb(0.75, 0.8, 0.88),
            ));
            root.spawn(fonted(
                art,
                "ENTER / TAP TO POUR AGAIN",
                24.0,
                Color::srgb(0.55, 0.95, 0.85),
            ));
            root.spawn(fonted(
                art,
                "ESC FOR THE TITLE SCREEN",
                16.0,
                Color::srgb(0.6, 0.64, 0.72),
            ));
        });
}

pub fn touch_mode_hint(touch: Res<PadInput>) -> bool {
    touch.touch_mode
}

pub fn theme_label(theme: usize) -> String {
    format!("{} · {}", THEME_NAMES[theme], THEME_SUBS[theme])
}
