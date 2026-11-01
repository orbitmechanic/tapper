//! A tiny auto-bartender used for screenshots, QA and the `--demo` flag.

use bevy::prelude::*;

use crate::AppState;
use crate::bonus::{Bonus, BonusPhase};
use crate::game::{Bartender, Game, Mug, MugKind, Patron, PatronState, Phase};
use crate::input::{Action, Actions};
use crate::world::{BT_KEG_SLOT, SPAWN_SLOT};

#[derive(Resource, Default)]
pub struct Demo {
    pub enabled: bool,
    pub time: f32,
    pub serve_cd: f32,
    pub start_cd: f32,
    pub bonus_cd: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn demo_drive(
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut demo: ResMut<Demo>,
    mut actions: ResMut<Actions>,
    game: Res<Game>,
    mut bt: Query<&mut Bartender>,
    patrons: Query<&Patron>,
    mugs: Query<&Mug>,
    bonus: Res<Bonus>,
) {
    if !demo.enabled {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    demo.time += dt;
    demo.serve_cd -= dt;
    demo.start_cd -= dt;
    demo.bonus_cd -= dt;

    match state.get() {
        AppState::Title => {
            if demo.start_cd <= 0.0 {
                actions.set_start();
                demo.start_cd = 4.0;
            }
        }
        AppState::GameOver => {
            if demo.start_cd <= 0.0 {
                actions.set_start();
                demo.start_cd = 4.0;
            }
        }
        AppState::Bonus => {
            if bonus.phase == BonusPhase::Pick && demo.bonus_cd <= 0.0 {
                let target = bonus.correct_slot();
                if bonus.selection != target {
                    actions.set(Action::Right, target > bonus.selection);
                    actions.set(Action::Left, target < bonus.selection);
                } else {
                    actions.set(Action::Pour, true);
                    demo.bonus_cd = 1.2;
                }
            }
        }
        AppState::InGame => {
            if game.phase != Phase::Normal {
                return;
            }
            let Ok(mut bartender) = bt.single_mut() else {
                return;
            };

            // Catch any empty mug first.
            let empty = mugs
                .iter()
                .filter(|m| m.kind == MugKind::Empty)
                .min_by(|a, b| a.slot.total_cmp(&b.slot));
            if let Some(mug) = empty {
                bartender.lane = mug.lane;
                bartender.slot = BT_KEG_SLOT;
                return;
            }

            // Only serve an advancing patron in a lane with no mug already in
            // flight. This mirrors what a careful human player does and keeps
            // unattended demo runs from throwing away lives.
            let busy_lane = |lane: usize| {
                mugs.iter()
                    .any(|m| m.lane == lane && m.kind == MugKind::Full)
            };
            let target = patrons
                .iter()
                .filter(|p| {
                    p.state == PatronState::Advancing
                        && p.slot > 1.0
                        && p.slot < SPAWN_SLOT
                        && !busy_lane(p.lane)
                })
                .min_by(|a, b| a.slot.total_cmp(&b.slot));
            if let Some(patron) = target {
                if bartender.lane != patron.lane {
                    bartender.lane = patron.lane;
                    bartender.slot = BT_KEG_SLOT;
                }
                if demo.serve_cd <= 0.0 {
                    actions.set(Action::Pour, true);
                    demo.serve_cd = 0.55;
                }
            }
        }
    }
}
