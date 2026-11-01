//! The between-theme shell game: one can is unshaken; follow it.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::AppState;
use crate::audio::Sfx;
use crate::game::{Game, PendingStage, Rng, ScreenEntity};
use crate::input::{Action, Actions};
use crate::render::Art;
use crate::world::BONUS_WIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusPhase {
    Shake,
    Shuffle,
    Pick,
    Reveal,
}

#[derive(Resource)]
pub struct Bonus {
    pub theme: usize,
    pub phase: BonusPhase,
    pub timer: f32,
    pub slots: [f32; 6],
    pub correct_can: usize,
    pub selection: usize,
    pub moves: Vec<(usize, usize)>,
    pub move_index: usize,
    pub move_timer: f32,
    pub result: Option<bool>,
    pub pending: Option<PendingStage>,
}

impl Default for Bonus {
    fn default() -> Self {
        Self {
            theme: 0,
            phase: BonusPhase::Shake,
            timer: 0.0,
            slots: [0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            correct_can: 0,
            selection: 2,
            moves: Vec::new(),
            move_index: 0,
            move_timer: 0.0,
            result: None,
            pending: None,
        }
    }
}

impl Bonus {
    pub fn reset(&mut self, theme: usize, pending: PendingStage, rng: &mut Rng) {
        self.theme = theme;
        self.phase = BonusPhase::Shake;
        self.timer = 1.9;
        self.slots = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        self.correct_can = rng.usize_below(6);
        self.selection = 2;
        self.moves = build_moves(rng, 6 + theme * 2);
        self.move_index = 0;
        self.move_timer = 0.0;
        self.result = None;
        self.pending = Some(pending);
    }

    pub fn correct_slot(&self) -> usize {
        self.slots[self.correct_can].round() as usize
    }

    pub fn can_slot(&self, can: usize) -> f32 {
        self.slots[can]
    }
}

pub fn build_moves(rng: &mut Rng, count: usize) -> Vec<(usize, usize)> {
    let mut moves = Vec::with_capacity(count);
    let mut last = usize::MAX;
    for _ in 0..count {
        let a = rng.usize_below(6);
        let mut b = rng.usize_below(5);
        if b >= a {
            b += 1;
        }
        if a == last {
            // Avoid immediately undoing the previous swap.
            if let Some(prev) = moves.last_mut() {
                *prev = (b, a);
                last = a;
                continue;
            }
        }
        moves.push((a, b));
        last = b;
    }
    moves
}

pub fn apply_swap(slots: &mut [f32; 6], a: usize, b: usize) {
    slots.swap(a, b);
}

pub fn spawn_bonus_scene(commands: &mut Commands, art: &Art, theme: usize) {
    commands.spawn((
        Sprite {
            image: art.background(theme),
            custom_size: Some(Vec2::new(320.0, 240.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -20.0),
        BonusEntity,
    ));
    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.01, 0.012, 0.02, 0.72),
            Vec2::new(320.0, 240.0),
        ),
        Transform::from_xyz(0.0, 0.0, -18.0),
        BonusEntity,
    ));

    // The masked shaker and the long table.
    commands.spawn((
        Sprite {
            image: art.shaker.clone(),
            custom_size: Some(Vec2::new(30.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(-102.0, -66.0, 0.0),
        Anchor::BOTTOM_CENTER,
        BonusShaker,
        BonusEntity,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.22, 0.13, 0.08), Vec2::new(250.0, 14.0)),
        Transform::from_xyz(10.0, -66.0, -2.0),
        BonusEntity,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.55, 0.32, 0.14), Vec2::new(250.0, 3.0)),
        Transform::from_xyz(10.0, -58.0, -1.0),
        BonusEntity,
    ));

    for i in 0..6 {
        commands.spawn((
            Sprite {
                image: art.can.clone(),
                custom_size: Some(Vec2::new(18.0, 27.0)),
                ..default()
            },
            Transform::from_xyz(can_x(i), -60.0, 2.0),
            Anchor::BOTTOM_CENTER,
            BonusCan(i),
            BonusEntity,
        ));
    }

    commands.spawn((
        Sprite {
            image: art.arrow.clone(),
            custom_size: Some(Vec2::new(16.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(can_x(2), -26.0, 5.0),
        BonusArrow,
        BonusEntity,
    ));
}

fn can_x(slot: usize) -> f32 {
    -55.0 + slot as f32 * 25.0
}

#[derive(Component)]
pub struct BonusEntity;

#[derive(Component)]
pub struct BonusCan(pub usize);

#[derive(Component)]
pub struct BonusArrow;

#[derive(Component)]
pub struct BonusShaker;

#[allow(clippy::too_many_arguments)]
pub fn enter_bonus(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
    mut bonus: ResMut<Bonus>,
    mut rng: ResMut<Rng>,
    old: Query<Entity, (With<ScreenEntity>, Without<BonusEntity>)>,
) {
    for e in &old {
        commands.entity(e).despawn();
    }
    let pending = game.pending.unwrap_or(PendingStage {
        next_screen: 0,
        next_loop: game.loop_no + 1,
    });
    let theme = game.theme();
    bonus.reset(theme, pending, &mut rng);
    game.pending = some_pending(&bonus);
    spawn_bonus_scene(&mut commands, &art, theme);
}

fn some_pending(bonus: &Bonus) -> Option<PendingStage> {
    bonus.pending
}

pub fn update_bonus(
    time: Res<Time>,
    mut bonus: ResMut<Bonus>,
    actions: Res<Actions>,
    mut game: ResMut<Game>,
    mut sfx: MessageWriter<Sfx>,
    mut next: ResMut<NextState<AppState>>,
) {
    let dt = time.delta_secs().min(0.05);
    match bonus.phase {
        BonusPhase::Shake => {
            bonus.timer -= dt;
            if bonus.timer <= 0.0 {
                bonus.phase = BonusPhase::Shuffle;
                bonus.move_timer = 0.55;
            }
        }
        BonusPhase::Shuffle => {
            bonus.move_timer -= dt;
            if bonus.move_timer <= 0.0 {
                if bonus.move_index < bonus.moves.len() {
                    let (a, b) = bonus.moves[bonus.move_index];
                    apply_swap(&mut bonus.slots, a, b);
                    bonus.move_index += 1;
                    bonus.move_timer = 0.30;
                } else {
                    bonus.phase = BonusPhase::Pick;
                    bonus.timer = 0.0;
                }
            }
        }
        BonusPhase::Pick => {
            if actions.just(Action::Left) {
                bonus.selection = (bonus.selection + 5) % 6;
            }
            if actions.just(Action::Right) {
                bonus.selection = (bonus.selection + 1) % 6;
            }
            if actions.just(Action::Pour) {
                let won = bonus.correct_slot() == bonus.selection;
                bonus.result = Some(won);
                bonus.phase = BonusPhase::Reveal;
                bonus.timer = 2.6;
                if won {
                    game.add_score(BONUS_WIN);
                    sfx.write(Sfx::BonusWin);
                } else {
                    sfx.write(Sfx::BonusLose);
                }
            }
        }
        BonusPhase::Reveal => {
            bonus.timer -= dt;
            if bonus.timer <= 0.0 {
                let Some(pending) = bonus.pending else {
                    return;
                };
                game.setup_screen(pending.next_screen, pending.next_loop);
                game.needs_load = true;
                next.set(AppState::InGame);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn sync_bonus(
    time: Res<Time>,
    bonus: Res<Bonus>,
    mut cans: Query<(&BonusCan, &mut Transform, &mut Sprite)>,
    mut arrow: Query<&mut Transform, (With<BonusArrow>, Without<BonusCan>, Without<BonusShaker>)>,
    mut shaker: Query<&mut Transform, (With<BonusShaker>, Without<BonusCan>, Without<BonusArrow>)>,
) {
    let t = time.elapsed_secs();
    for (can, mut tf, mut sprite) in &mut cans {
        let slot = bonus.can_slot(can.0);
        tf.translation.x = can_x(0) + slot * 25.0;
        let wobble = if bonus.phase == BonusPhase::Shake && can.0 != bonus.correct_can {
            (t * 26.0 + can.0 as f32 * 2.1).sin() * 2.4
        } else {
            0.0
        };
        tf.translation.x += wobble;
        tf.translation.y = -60.0;
        if bonus.phase == BonusPhase::Reveal && bonus.result.is_some() && can.0 == bonus.correct_can
        {
            let pulse = 1.0 + (t * 12.0).sin().abs() * 0.18;
            sprite.custom_size = Some(Vec2::new(18.0 * pulse, 27.0 * pulse));
        } else {
            sprite.custom_size = Some(Vec2::new(18.0, 27.0));
        }
    }
    for mut tf in &mut arrow {
        tf.translation.x = can_x(0) + bonus.selection as f32 * 25.0;
        tf.translation.y = -27.0 + (t * 6.0).sin() * 1.8;
    }
    for mut tf in &mut shaker {
        tf.translation.x = -102.0
            + if bonus.phase == BonusPhase::Shake {
                (t * 18.0).sin() * 2.0
            } else {
                0.0
            };
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum BonusUi {
    Prompt,
    Score,
}

pub fn spawn_bonus_ui_system(mut commands: Commands, art: Res<Art>) {
    spawn_bonus_ui(&mut commands, &art);
}

pub fn spawn_bonus_ui(commands: &mut Commands, art: &Art) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit(AppState::Bonus),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(190.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.015, 0.018, 0.024)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(150.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.015, 0.018, 0.024)),
            ));
            root.spawn((
                Text::new("SHELL GAME"),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: FontSize::Px(46.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.3)),
                TextShadow {
                    offset: Vec2::new(3.0, 3.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.8),
                },
                Node {
                    margin: UiRect {
                        top: Val::Px(46.0),
                        ..default()
                    },
                    ..default()
                },
            ));
            root.spawn((
                Text::new("ONE CAN IS STILL. FOLLOW IT."),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: FontSize::Px(17.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.92, 0.84)),
            ));
            root.spawn((
                Text::new("WATCH THE CANS"),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: FontSize::Px(26.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.94, 0.8)),
                Node {
                    margin: UiRect {
                        top: Val::Px(430.0),
                        ..default()
                    },
                    ..default()
                },
                BonusUi::Prompt,
            ));
            root.spawn((
                Text::new(""),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: FontSize::Px(18.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.7, 0.3)),
                BonusUi::Score,
            ));
        });
}

pub fn update_bonus_ui(bonus: Res<Bonus>, game: Res<Game>, mut q: Query<(&BonusUi, &mut Text)>) {
    for (kind, mut text) in &mut q {
        match kind {
            BonusUi::Prompt => {
                text.0 = match bonus.phase {
                    BonusPhase::Shake => "HE'S SHAKING FIVE OF THEM…".into(),
                    BonusPhase::Shuffle => "KEEP YOUR EYE ON THE STILL ONE…".into(),
                    BonusPhase::Pick => "<  POUR TO PICK  >".into(),
                    BonusPhase::Reveal => {
                        if bonus.result == Some(true) {
                            format!("STILL!  +{BONUS_WIN}")
                        } else {
                            "FIZZ IN THE FACE!".to_string()
                        }
                    }
                };
            }
            BonusUi::Score => {
                text.0 = format!("SCORE  {:06}", game.score);
            }
        }
    }
}

pub fn exit_bonus(mut commands: Commands, entities: Query<Entity, With<BonusEntity>>) {
    for e in &entities {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_track_the_still_can() {
        let mut bonus = Bonus::default();
        let mut rng = Rng::default();
        bonus.reset(
            1,
            PendingStage {
                next_screen: 5,
                next_loop: 0,
            },
            &mut rng,
        );
        let still = bonus.correct_can;
        for (a, b) in bonus.moves.clone() {
            apply_swap(&mut bonus.slots, a, b);
        }
        assert_eq!(bonus.slots[still].round() as usize, bonus.correct_slot());
        assert!((0..6).contains(&bonus.correct_slot()));
    }

    #[test]
    fn moves_never_swap_a_can_with_itself() {
        let mut rng = Rng::default();
        for _ in 0..50 {
            for (a, b) in build_moves(&mut rng, 10) {
                assert_ne!(a, b);
                assert!(a < 6 && b < 6);
            }
        }
    }
}
