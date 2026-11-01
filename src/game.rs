//! Core Tapper simulation: bartender, patrons, mugs, tips and screen flow.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::audio::Sfx;
use crate::fx::{Fx, spawn_burst};
use crate::input::{Action, Actions};
use crate::world::*;

pub const POUR_TIME: f32 = 0.34;
const BT_RUN_SPEED: f32 = 3.4;
const SLIDE_TIME: f32 = 0.42;
const TIP_TTL: f32 = 11.0;
const DANCE_TIME: f32 = 4.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Normal,
    Dying,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatronState {
    Advancing,
    Drinking,
    Exiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MugKind {
    Full,
    Empty,
}

#[derive(Component)]
pub struct Patron {
    pub lane: usize,
    pub slot: f32,
    pub kind: u8,
    pub variant: u8,
    pub state: PatronState,
    pub timer: f32,
    pub drink_total: f32,
    pub slide_from: f32,
    pub target_slot: f32,
    pub anim: f32,
    pub distracted: bool,
}

#[derive(Component)]
pub struct Mug {
    pub lane: usize,
    pub slot: f32,
    pub kind: MugKind,
}

#[derive(Component)]
pub struct Tip {
    pub lane: usize,
    pub slot: f32,
    pub ttl: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Bartender {
    pub lane: usize,
    pub slot: f32,
    pub pour: f32,
    pub facing: f32,
    pub anim: f32,
}

#[derive(Component)]
pub struct Dancer;

/// Everything spawned for the active screen, despawned when it reloads.
#[derive(Component, Default)]
pub struct ScreenEntity;

#[derive(Component)]
pub struct GameEntity;

#[derive(Clone, Copy, Debug, Default)]
pub struct LaneState {
    pub remaining: u8,
    pub spawn_timer: f32,
    pub keg_right: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingStage {
    pub next_screen: usize,
    pub next_loop: u32,
}

#[derive(Resource)]
pub struct Game {
    pub screen: usize,
    pub start_screen: usize,
    pub loop_no: u32,
    pub phase: Phase,
    pub phase_timer: f32,
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub next_extra: u32,
    pub banner: String,
    pub banner_timer: f32,
    pub dancers: f32,
    pub lanes: [LaneState; LANE_COUNT],
    pub needs_load: bool,
    pub fresh: bool,
    pub pending: Option<PendingStage>,
    pub screen_age: f32,
    pub muted: bool,
    pub started: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            screen: 0,
            start_screen: 0,
            loop_no: 0,
            phase: Phase::Normal,
            phase_timer: 0.0,
            score: 0,
            lives: 3,
            high_score: 0,
            next_extra: 20_000,
            banner: String::new(),
            banner_timer: 0.0,
            dancers: 0.0,
            lanes: [LaneState::default(); LANE_COUNT],
            needs_load: true,
            fresh: true,
            pending: None,
            screen_age: 0.0,
            muted: false,
            started: false,
        }
    }
}

impl Game {
    pub fn reset_run(&mut self) {
        let high = self.high_score;
        let muted = self.muted;
        let start = self.start_screen;
        *self = Self::default();
        self.high_score = high;
        self.muted = muted;
        self.start_screen = start;
        self.fresh = false;
        self.started = true;
        self.setup_screen(start, 0);
    }

    pub fn theme(&self) -> usize {
        theme_for_screen(self.screen)
    }

    pub fn difficulty(&self) -> Difficulty {
        difficulty(self.screen, self.loop_no)
    }

    pub fn setup_screen(&mut self, screen: usize, loop_no: u32) {
        self.screen = screen;
        self.loop_no = loop_no;
        let plan = screen_plan(screen);
        for (lane, p) in plan.iter().enumerate() {
            self.lanes[lane] = LaneState {
                remaining: p.count,
                spawn_timer: 0.35 + lane as f32 * 0.42,
                keg_right: p.keg_right,
            };
        }
        self.phase = Phase::Normal;
        self.phase_timer = 0.0;
        self.screen_age = 0.0;
        self.dancers = 0.0;
        self.pending = None;
        self.show_banner(
            format!(
                "SCREEN {} · {}",
                screen + 1,
                THEME_NAMES[theme_for_screen(screen)]
            ),
            2.2,
        );
    }

    pub fn show_banner(&mut self, text: impl Into<String>, seconds: f32) {
        self.banner = text.into();
        self.banner_timer = seconds;
    }

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
        if self.score >= self.next_extra {
            self.lives += 1;
            self.next_extra += 60_000;
            self.show_banner("EXTRA SHIFT · ONE MORE LIFE", 2.4);
        }
        self.high_score = self.high_score.max(self.score);
    }

    pub fn lane_active(&self, patrons: &[(Entity, &Patron)], lane: usize) -> usize {
        patrons.iter().filter(|(_, p)| p.lane == lane).count()
    }
}

#[derive(Resource)]
pub struct Rng(pub u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0x9e37_79b9_7f4a_7c15)
    }
}

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    pub fn usize_below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            self.next_u32() as usize % n
        }
    }
}

pub fn spawn_bartender(commands: &mut Commands) {
    commands.spawn((
        Bartender {
            lane: 0,
            slot: BT_KEG_SLOT,
            pour: 0.0,
            facing: 1.0,
            anim: 0.0,
        },
        Sprite::default(),
        Transform::from_xyz(slot_x(true, BT_KEG_SLOT), LANE_Y[0] + 1.0, 3.0),
        Anchor::BOTTOM_CENTER,
        ScreenEntity,
    ));
}

/// Advances phase timers, banners and screen transitions.
#[allow(clippy::too_many_arguments)]
pub fn tick_phase(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<crate::AppState>>,
    mut sfx: MessageWriter<Sfx>,
    patrons: Query<&Patron>,
    mugs: Query<&Mug>,
) {
    let dt = time.delta_secs().min(0.05);
    game.banner_timer = (game.banner_timer - dt).max(0.0);
    if game.banner_timer <= 0.0 {
        game.banner.clear();
    }
    game.dancers = (game.dancers - dt).max(0.0);

    match game.phase {
        Phase::Normal => {
            game.screen_age += dt;
            if game.screen_age > 1.0
                && patrons.is_empty()
                && mugs.is_empty()
                && game.lanes.iter().all(|l| l.remaining == 0)
            {
                game.phase = Phase::Cleared;
                game.phase_timer = 2.4;
                game.add_score(SCREEN_CLEAR_BONUS);
                game.show_banner(format!("SCREEN CLEAR  +{SCREEN_CLEAR_BONUS}"), 2.4);
                sfx.write(Sfx::LevelClear);
                let next_index = next_screen(game.screen);
                game.pending = if milestone_after(game.screen) {
                    Some(PendingStage {
                        next_screen: next_index.0,
                        next_loop: game.loop_no + next_index.1,
                    })
                } else {
                    None
                };
            }
        }
        Phase::Dying => {
            game.phase_timer -= dt;
            if game.phase_timer <= 0.0 {
                if game.lives == 0 {
                    game.high_score = game.high_score.max(game.score);
                    game.phase = Phase::Normal;
                    game.phase_timer = 0.0;
                    sfx.write(Sfx::GameOver);
                    next.set(crate::AppState::GameOver);
                } else {
                    game.needs_load = true;
                    game.show_banner("BACK TO WORK", 1.6);
                }
            }
        }
        Phase::Cleared => {
            game.phase_timer -= dt;
            if game.phase_timer <= 0.0 {
                if let Some(pending) = game.pending {
                    next.set(crate::AppState::Bonus);
                    game.pending = Some(pending);
                } else {
                    let (screen, loop_delta) = next_screen(game.screen);
                    let next_loop = game.loop_no + loop_delta;
                    game.setup_screen(screen, next_loop);
                    game.needs_load = true;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn bartender_input(
    time: Res<Time>,
    actions: Res<Actions>,
    game: Res<Game>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
    mut q: Query<&mut Bartender>,
) {
    if game.phase != Phase::Normal {
        return;
    }
    let Ok(mut bt) = q.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let plan = screen_plan(game.screen);
    let keg_right = plan[bt.lane].keg_right;
    bt.anim += dt;

    if actions.just(Action::Pour) && bt.pour <= 0.0 {
        bt.slot = BT_KEG_SLOT;
        bt.pour = POUR_TIME;
        sfx.write(Sfx::Pour);
    }

    if bt.pour > 0.0 {
        bt.pour -= dt;
        if bt.pour <= 0.0 {
            bt.pour = 0.0;
            commands.spawn((
                Mug {
                    lane: bt.lane,
                    slot: POUR_SLOT,
                    kind: MugKind::Full,
                },
                Sprite::default(),
                Transform::from_xyz(slot_x(keg_right, POUR_SLOT), LANE_Y[bt.lane] + 7.0, 5.0),
                Anchor::CENTER,
                ScreenEntity,
            ));
            sfx.write(Sfx::Serve);
        }
        return;
    }

    if actions.just(Action::Up) {
        bt.lane = (bt.lane + LANE_COUNT - 1) % LANE_COUNT;
        bt.slot = BT_KEG_SLOT;
        bt.facing = 1.0;
        sfx.write(Sfx::Lane);
    } else if actions.just(Action::Down) {
        bt.lane = (bt.lane + 1) % LANE_COUNT;
        bt.slot = BT_KEG_SLOT;
        bt.facing = 1.0;
        sfx.write(Sfx::Lane);
    }

    let keg_right = screen_plan(game.screen)[bt.lane].keg_right;
    let mut h = 0.0;
    if actions.pressed(Action::Left) {
        h -= 1.0;
    }
    if actions.pressed(Action::Right) {
        h += 1.0;
    }
    if h != 0.0 {
        let delta = -along_bar_sign(keg_right) * h * BT_RUN_SPEED * dt;
        bt.slot = (bt.slot + delta).clamp(BT_KEG_SLOT - 0.1, BT_MAX_SLOT);
        bt.facing = if delta > 0.0 {
            -if keg_right { 1.0 } else { -1.0 }
        } else {
            if keg_right { 1.0 } else { -1.0 }
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_patrons(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
    mut fx: ResMut<Fx>,
    mut q: Query<(Entity, &mut Patron)>,
    mut bartender: Query<&mut Bartender>,
) {
    let dt = time.delta_secs().min(0.05);
    if game.phase != Phase::Normal {
        return;
    }
    let diff = game.difficulty();
    let mut grabbed = false;
    let mut served: Vec<(Entity, u32, usize)> = Vec::new();

    for (entity, mut p) in &mut q {
        match p.state {
            PatronState::Advancing => {
                p.distracted = game.dancers > 0.0;
                if p.distracted {
                    p.anim += dt * 1.5;
                } else {
                    p.slot -= diff.advance_speed * dt;
                    p.anim += dt * (1.4 + diff.advance_speed * 4.0);
                    if p.slot <= GRAB_SLOT {
                        grabbed = true;
                    }
                }
            }
            PatronState::Drinking => {
                p.timer -= dt;
                let elapsed = (p.drink_total - p.timer).max(0.0);
                if elapsed < SLIDE_TIME {
                    let t = (elapsed / SLIDE_TIME).clamp(0.0, 1.0);
                    let eased = t * t * (3.0 - 2.0 * t);
                    p.slot = p.slide_from + (p.target_slot - p.slide_from) * eased;
                } else {
                    p.slot = p.target_slot;
                }
                p.anim += dt * 2.0;
                if p.timer <= 0.0 {
                    if p.target_slot >= DOOR_SLOT {
                        p.state = PatronState::Exiting;
                        p.timer = 0.22;
                        p.slot = p.target_slot;
                    } else {
                        p.state = PatronState::Advancing;
                        p.target_slot = p.slot;
                        p.slide_from = p.slot;
                    }
                }
            }
            PatronState::Exiting => {
                p.slot += 0.9 * dt;
                p.timer -= dt;
                if p.timer <= 0.0 {
                    served.push((entity, score_for(game.theme()), p.lane));
                }
            }
        }
    }

    for (entity, points, lane) in served {
        commands.entity(entity).despawn();
        game.add_score(points);
        sfx.write(Sfx::Served);
        spawn_burst(
            &mut commands,
            Vec2::new(
                slot_x(screen_plan(game.screen)[lane].keg_right, 9.7),
                LANE_Y[lane] + 12.0,
            ),
            Color::srgb(1.0, 0.86, 0.4),
            5,
            40.0,
        );
    }

    if grabbed && game.phase == Phase::Normal {
        game.lives = game.lives.saturating_sub(1);
        game.phase = Phase::Dying;
        game.phase_timer = 1.75;
        game.show_banner("LAST CALL! THEY GOT YOU", 1.7);
        game.dancers = 0.0;
        fx.shake = 7.0;
        sfx.write(Sfx::Grab);
        if let Ok(mut bt) = bartender.single_mut() {
            bt.pour = 0.0;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_mugs(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
    mut fx: ResMut<Fx>,
    mut rng: ResMut<Rng>,
    mut mugs: Query<(Entity, &mut Mug)>,
    mut patrons: Query<(Entity, &mut Patron)>,
    bartender: Query<&Bartender>,
    tips: Query<&Tip>,
) {
    if game.phase != Phase::Normal {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let diff = game.difficulty();
    let bt = bartender.single().ok().copied();
    let snapshot: Vec<(Entity, usize, f32, PatronState, bool)> = patrons
        .iter()
        .map(|(e, p)| (e, p.lane, p.slot, p.state, p.distracted))
        .collect();
    let mut occupied: HashSet<Entity> = HashSet::new();
    let mut hits: Vec<(Entity, Entity, usize)> = Vec::new();
    let mut broken: Vec<(Entity, Vec2)> = Vec::new();
    let mut caught_empty = 0u32;
    let mut lost_life = false;

    for (mug_entity, mut mug) in &mut mugs {
        match mug.kind {
            MugKind::Full => {
                let new_slot = mug.slot + diff.serve_speed * dt;
                let mut catcher: Option<(Entity, f32)> = None;
                for (pe, lane, slot, state, distracted) in &snapshot {
                    if *lane != mug.lane
                        || *state != PatronState::Advancing
                        || *distracted
                        || occupied.contains(pe)
                    {
                        continue;
                    }
                    if *slot >= mug.slot - 0.25
                        && *slot <= new_slot + 0.55
                        && catcher.is_none_or(|(_, best)| *slot < best)
                    {
                        catcher = Some((*pe, *slot));
                    }
                }
                if let Some((pe, _)) = catcher {
                    occupied.insert(pe);
                    hits.push((mug_entity, pe, mug.lane));
                } else if new_slot >= DOOR_SLOT + 0.45 {
                    let x = slot_x(
                        screen_plan(game.screen)[mug.lane].keg_right,
                        DOOR_SLOT + 0.2,
                    );
                    broken.push((mug_entity, Vec2::new(x, LANE_Y[mug.lane] + 7.0)));
                    lost_life = true;
                } else {
                    mug.slot = new_slot;
                }
            }
            MugKind::Empty => {
                let new_slot = mug.slot - diff.empty_speed * dt;
                let catch_ok = bt.is_some_and(|b| {
                    b.lane == mug.lane && can_catch_empty(b.slot) && new_slot <= 0.62
                });
                if catch_ok && new_slot <= 0.62 {
                    caught_empty += EMPTY_MUG_BONUS;
                    sfx.write(Sfx::EmptyCatch);
                    commands.entity(mug_entity).despawn();
                } else if new_slot <= 0.15 {
                    let x = slot_x(screen_plan(game.screen)[mug.lane].keg_right, 0.0);
                    broken.push((mug_entity, Vec2::new(x, LANE_Y[mug.lane] + 7.0)));
                    lost_life = true;
                } else {
                    mug.slot = new_slot;
                }
            }
        }
    }

    for (mug_entity, patron_entity, lane) in hits {
        commands.entity(mug_entity).despawn();
        let diff = game.difficulty();
        if let Ok((_, mut p)) = patrons.get_mut(patron_entity) {
            p.state = PatronState::Drinking;
            p.slide_from = p.slot;
            p.target_slot = (p.slot + diff.push_slots).min(DOOR_SLOT + 0.35);
            p.drink_total = SLIDE_TIME + diff.drink_time;
            p.timer = p.drink_total;
            p.anim = 0.0;
            let target = p.target_slot;
            sfx.write(Sfx::Catch);
            spawn_burst(
                &mut commands,
                Vec2::new(
                    slot_x(screen_plan(game.screen)[lane].keg_right, p.slot),
                    LANE_Y[lane] + 14.0,
                ),
                Color::srgb(1.0, 0.94, 0.62),
                4,
                30.0,
            );
            if target < DOOR_SLOT {
                commands.spawn((
                    Mug {
                        lane,
                        slot: p.slot + 0.35,
                        kind: MugKind::Empty,
                    },
                    Sprite::default(),
                    Transform::from_xyz(
                        slot_x(screen_plan(game.screen)[lane].keg_right, p.slot + 0.35),
                        LANE_Y[lane] + 7.0,
                        4.0,
                    ),
                    Anchor::CENTER,
                    ScreenEntity,
                ));
                let lane_tips = tips.iter().filter(|t| t.lane == lane).count();
                if lane_tips == 0 && rng.chance(0.17) {
                    let tip_slot = (p.slot + rng.range(1.6, 3.0)).min(DOOR_SLOT - 0.35);
                    commands.spawn((
                        Tip {
                            lane,
                            slot: tip_slot,
                            ttl: TIP_TTL,
                        },
                        Sprite::default(),
                        Transform::from_xyz(
                            slot_x(screen_plan(game.screen)[lane].keg_right, tip_slot),
                            LANE_Y[lane] + 9.0,
                            2.0,
                        ),
                        Anchor::CENTER,
                        ScreenEntity,
                    ));
                }
            }
        }
    }

    if caught_empty > 0 {
        game.add_score(caught_empty);
    }

    for (entity, at) in broken {
        commands.entity(entity).despawn();
        sfx.write(Sfx::Break);
        fx.shake = 5.0;
        spawn_burst(&mut commands, at, Color::srgb(0.85, 0.93, 1.0), 12, 75.0);
    }

    if lost_life && game.phase == Phase::Normal {
        game.lives = game.lives.saturating_sub(1);
        game.phase = Phase::Dying;
        game.phase_timer = 1.75;
        game.show_banner("LAST CALL! A MUG BIT IT", 1.7);
        game.dancers = 0.0;
        sfx.write(Sfx::Grab);
    }
}

pub fn update_tips(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
    mut q: Query<(Entity, &mut Tip)>,
    bartender: Query<&Bartender>,
) {
    if game.phase != Phase::Normal {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let Ok(bt) = bartender.single() else {
        return;
    };
    for (entity, mut tip) in &mut q {
        tip.ttl -= dt;
        if tip.ttl <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if tip.lane == bt.lane && (tip.slot - bt.slot).abs() < 0.72 {
            commands.entity(entity).despawn();
            game.add_score(TIP_BONUS);
            game.dancers = DANCE_TIME;
            game.show_banner(format!("TIP!  +{TIP_BONUS}"), 1.8);
            sfx.write(Sfx::Tip);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_patrons(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    patrons: Query<&Patron>,
) {
    if game.phase != Phase::Normal {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let diff = game.difficulty();
    let active_counts: [usize; LANE_COUNT] =
        std::array::from_fn(|lane| patrons.iter().filter(|p| p.lane == lane).count());

    for lane in 0..LANE_COUNT {
        if game.lanes[lane].remaining == 0 {
            continue;
        }
        if active_counts[lane] >= diff.max_active {
            continue;
        }
        game.lanes[lane].spawn_timer -= dt;
        if game.lanes[lane].spawn_timer > 0.0 {
            continue;
        }
        let keg_right = game.lanes[lane].keg_right;
        let variant = rng.usize_below(4) as u8;
        commands.spawn((
            Patron {
                lane,
                slot: SPAWN_SLOT,
                kind: variant,
                variant,
                state: PatronState::Advancing,
                timer: 0.0,
                drink_total: 0.0,
                slide_from: SPAWN_SLOT,
                target_slot: SPAWN_SLOT,
                anim: 0.0,
                distracted: false,
            },
            Sprite::default(),
            Transform::from_xyz(slot_x(keg_right, SPAWN_SLOT), LANE_Y[lane] + 1.0, 2.0),
            Anchor::BOTTOM_CENTER,
            ScreenEntity,
        ));
        game.lanes[lane].remaining -= 1;
        game.lanes[lane].spawn_timer = diff.spawn_interval * rng.range(0.82, 1.22);
        info!(
            "spawn patron lane {} variant {} slot {:.2} remaining {}",
            lane, variant, SPAWN_SLOT, game.lanes[lane].remaining
        );
    }
}

/// Ends the current run when lives hit zero (called from `tick_phase`).
pub fn finish_run(game: &mut Game) {
    game.high_score = game.high_score.max(game.score);
    game.phase = Phase::Normal;
    game.phase_timer = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_rolls_extra_lives() {
        let mut game = Game::default();
        game.add_score(19_990);
        assert_eq!(game.lives, 3);
        assert_eq!(game.next_extra, 20_000);
        game.add_score(20);
        assert_eq!(game.lives, 4);
        assert_eq!(game.next_extra, 80_000);
    }

    #[test]
    fn setup_screen_loads_counts_and_sides() {
        let mut game = Game::default();
        game.setup_screen(5, 0);
        assert_eq!(game.lanes.map(|l| l.remaining), [2, 3, 3, 4]);
        assert_eq!(game.lanes.map(|l| l.keg_right), [false, false, true, true]);
        assert_eq!(game.theme(), 2);
        assert_eq!(game.phase, Phase::Normal);
    }

    #[test]
    fn reset_keeps_high_score_and_mute() {
        let mut game = Game {
            score: 12_345,
            high_score: 99_999,
            muted: true,
            ..Default::default()
        };
        game.reset_run();
        assert_eq!(game.score, 0);
        assert_eq!(game.lives, 3);
        assert_eq!(game.high_score, 99_999);
        assert!(game.muted);
        assert_eq!(game.screen, 0);
        assert!(!game.fresh);
    }
}
