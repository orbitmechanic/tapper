use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use tapper::AppState;
use tapper::audio::Sfx;
use tapper::fx::Fx;
use tapper::game::*;
use tapper::input::Actions;
use tapper::world::*;

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_message::<Sfx>()
        .init_resource::<Game>()
        .init_resource::<Rng>()
        .init_resource::<Fx>()
        .init_resource::<Actions>()
        .init_state::<AppState>();
    app
}

fn advance(app: &mut App, secs: f32) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        secs,
    )));
    app.update();
}

fn bartender(app: &mut App, lane: usize) {
    app.world_mut().spawn(Bartender {
        lane,
        slot: BT_KEG_SLOT,
        pour: 0.0,
        facing: 1.0,
        anim: 0.0,
    });
}

fn patron(app: &mut App, lane: usize, slot: f32) -> Entity {
    app.world_mut()
        .spawn(Patron {
            lane,
            slot,
            kind: 1,
            variant: 1,
            state: PatronState::Advancing,
            timer: 0.0,
            drink_total: 0.0,
            slide_from: slot,
            target_slot: slot,
            anim: 0.0,
            distracted: false,
        })
        .id()
}

fn mug(app: &mut App, lane: usize, slot: f32, kind: MugKind) -> Entity {
    app.world_mut().spawn(Mug { lane, slot, kind }).id()
}

#[test]
fn full_mug_is_caught_by_first_patron() {
    let mut app = base_app();
    app.add_systems(Update, (update_mugs, update_patrons).chain());
    bartender(&mut app, 0);
    let p = patron(&mut app, 0, 4.0);
    let m = mug(&mut app, 0, 3.55, MugKind::Full);

    advance(&mut app, 0.05);

    assert!(app.world().get_entity(m).is_err(), "mug consumed");
    assert_eq!(
        app.world().get::<Patron>(p).unwrap().state,
        PatronState::Drinking
    );
    assert!(app.world().get::<Patron>(p).unwrap().target_slot > 4.0);
}

#[test]
fn patron_pushed_through_the_door_is_scored() {
    let mut app = base_app();
    app.add_systems(Update, (update_mugs, update_patrons).chain());
    bartender(&mut app, 0);
    let p = patron(&mut app, 0, 8.2);
    mug(&mut app, 0, 7.9, MugKind::Full);

    advance(&mut app, 0.05);
    for _ in 0..60 {
        advance(&mut app, 0.05);
    }

    assert!(app.world().get_entity(p).is_err(), "served patron left");
    let score = app.world().resource::<Game>().score;
    assert_eq!(score, score_for(0));
}

#[test]
fn empty_mug_near_the_keg_is_caught_for_points() {
    let mut app = base_app();
    app.add_systems(Update, (update_mugs, update_patrons).chain());
    bartender(&mut app, 0);
    let m = mug(&mut app, 0, 0.4, MugKind::Empty);

    for _ in 0..6 {
        advance(&mut app, 0.02);
    }

    assert!(app.world().get_entity(m).is_err());
    assert_eq!(app.world().resource::<Game>().score, EMPTY_MUG_BONUS);
    assert_eq!(app.world().resource::<Game>().lives, 3);
}

#[test]
fn dropped_empty_mug_costs_a_life() {
    let mut app = base_app();
    app.add_systems(Update, (update_mugs, update_patrons).chain());
    bartender(&mut app, 2);
    let m = mug(&mut app, 0, 0.4, MugKind::Empty);

    for _ in 0..8 {
        advance(&mut app, 0.02);
    }

    assert!(app.world().get_entity(m).is_err());
    assert_eq!(app.world().resource::<Game>().lives, 2);
    assert_eq!(app.world().resource::<Game>().phase, Phase::Dying);
}

#[test]
fn full_mug_that_reaches_the_door_breaks() {
    let mut app = base_app();
    app.add_systems(Update, update_mugs);
    bartender(&mut app, 0);
    let m = mug(&mut app, 0, DOOR_SLOT + 0.4, MugKind::Full);

    advance(&mut app, 0.05);
    advance(&mut app, 0.05);

    assert!(app.world().get_entity(m).is_err());
    assert_eq!(app.world().resource::<Game>().lives, 2);
    assert_eq!(app.world().resource::<Game>().phase, Phase::Dying);
}

#[test]
fn patron_reaching_the_keg_costs_a_life() {
    let mut app = base_app();
    app.add_systems(Update, update_patrons);
    bartender(&mut app, 0);
    let p = patron(&mut app, 0, GRAB_SLOT - 0.01);

    advance(&mut app, 0.05);

    assert!(app.world().get_entity(p).is_ok());
    assert_eq!(app.world().resource::<Game>().lives, 2);
    assert_eq!(app.world().resource::<Game>().phase, Phase::Dying);
}

#[test]
fn tips_pay_out_and_start_the_dancers() {
    let mut app = base_app();
    app.add_systems(Update, update_tips);
    bartender(&mut app, 1);
    let t = app
        .world_mut()
        .spawn(Tip {
            lane: 1,
            slot: BT_KEG_SLOT + 0.2,
            ttl: 3.0,
        })
        .id();

    advance(&mut app, 0.05);

    assert!(app.world().get_entity(t).is_err());
    let game = app.world().resource::<Game>();
    assert_eq!(game.score, TIP_BONUS);
    assert!(game.dancers > 4.0);
}

#[test]
fn clearing_the_last_patron_starts_the_clear_phase() {
    let mut app = base_app();
    app.add_systems(Update, tick_phase);
    {
        let mut game = app.world_mut().resource_mut::<Game>();
        game.screen = 1;
        game.screen_age = 2.0;
        game.lanes = [LaneState::default(); LANE_COUNT];
    }

    advance(&mut app, 0.05);

    let game = app.world().resource::<Game>();
    assert_eq!(game.phase, Phase::Cleared);
    assert_eq!(game.score, SCREEN_CLEAR_BONUS);
    assert!(game.pending.is_some(), "saloon clears into the bonus game");
}
