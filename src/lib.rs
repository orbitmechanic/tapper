//! TAPPER: LAST CALL — app wiring, states and capture harness.

pub mod audio;
pub mod bonus;
pub mod demo;
pub mod fx;
pub mod game;
pub mod input;
pub mod mobile;
pub mod render;
pub mod ui;
pub mod world;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::audio::{GlobalVolume, Volume};
use bevy::camera::ScalingMode;
use bevy::image::ImagePlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::sprite::Anchor;
use bevy::ui_render::prelude::UiAntiAlias;
use bevy::window::WindowResolution;

use audio::*;
use bonus::*;
use demo::*;
use fx::*;
use game::*;
use input::*;
use render::*;
use ui::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Title,
    InGame,
    Bonus,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(Resource, Default)]
pub struct CaptureConfig {
    pub screenshot: Option<String>,
    pub frame: u32,
    pub start: bool,
    pub demo: bool,
    pub screen: Option<usize>,
    pub bonus: bool,
    pub gameover: bool,
    pub mug_break: bool,
}

pub fn run() {
    let capture = parse_args();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TAPPER: LAST CALL".into(),
                    resolution: WindowResolution::new(960, 720).with_scale_factor_override(1.0),
                    resizable: false,
                    canvas: Some("#game-canvas".into()),
                    fit_canvas_to_parent: false,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.012, 0.01, 0.018)))
    .insert_resource(GlobalVolume::new(Volume::Linear(0.9)))
    .insert_resource(capture)
    .init_resource::<Game>()
    .init_resource::<Rng>()
    .init_resource::<Actions>()
    .init_resource::<PadInput>()
    .init_resource::<Fx>()
    .init_resource::<Paused>()
    .init_resource::<Demo>()
    .init_resource::<Bonus>()
    .init_resource::<MusicState>()
    .init_state::<AppState>()
    .add_message::<Sfx>()
    .add_systems(PreStartup, load_art)
    .add_systems(Startup, (setup_camera, load_sounds, apply_capture))
    .add_systems(OnEnter(AppState::InGame), (begin_session, spawn_hud_system))
    .add_systems(OnEnter(AppState::GameOver), spawn_end_screen)
    .add_systems(
        OnEnter(AppState::Bonus),
        (enter_bonus, spawn_bonus_ui_system).chain(),
    )
    .add_systems(
        Update,
        mug_break_scenario.run_if(in_state(AppState::InGame)),
    )
    .add_systems(
        Update,
        (
            poll_touch,
            update_actions,
            demo_drive,
            meta_input,
            play_sfx,
            update_music,
            apply_mute,
            (
                tick_phase,
                load_screen,
                bartender_input,
                spawn_patrons,
                update_patrons,
                update_mugs,
                update_tips,
            )
                .chain()
                .run_if(in_state(AppState::InGame))
                .run_if(not_paused),
            update_hud.run_if(in_state(AppState::InGame)),
            toggle_pause_veil.run_if(in_state(AppState::InGame)),
            update_bonus.run_if(in_state(AppState::Bonus)),
            publish_state_bridge,
            (
                sync_sprites.run_if(in_state(AppState::InGame)),
                sync_bonus.run_if(in_state(AppState::Bonus)),
                update_bonus_ui.run_if(in_state(AppState::Bonus)),
                ensure_title.run_if(in_state(AppState::Title)),
                update_title.run_if(in_state(AppState::Title)),
                update_particles,
                decay_fx,
                camera_shake,
            ),
            capture_system,
            clear_pulses,
        )
            .chain(),
    )
    .run();
}

#[allow(clippy::too_many_arguments)]
fn publish_state_bridge(
    game: Res<Game>,
    bt: Query<&Bartender>,
    patrons: Query<&Patron>,
    mugs: Query<&Mug>,
) {
    let (lane, slot) = bt.single().map(|b| (b.lane, b.slot)).unwrap_or((0, 0.0));
    let phase = match game.phase {
        Phase::Normal => 0,
        Phase::Dying => 1,
        Phase::Cleared => 2,
    };
    mobile::publish_state(
        game.score,
        game.lives,
        game.screen,
        game.loop_no,
        phase,
        lane,
        slot,
        patrons.iter().count(),
        mugs.iter().count(),
    );
}

/// Reproduction hook for the dropped-mug life-loss path: clear the current
/// patrons, then slide a full mug off the door so the death/respawn transition
/// runs in the real renderer. Used by `--mug-break` QA captures.
fn mug_break_scenario(
    capture: Res<CaptureConfig>,
    mut done: Local<bool>,
    game: Res<Game>,
    patrons: Query<Entity, With<Patron>>,
    mut commands: Commands,
) {
    if !capture.mug_break || *done || game.screen_age < 0.5 {
        return;
    }
    *done = true;
    for entity in &patrons {
        commands.entity(entity).despawn();
    }
    let keg_right = world::screen_plan(game.screen)[0].keg_right;
    commands.spawn((
        Mug {
            lane: 0,
            slot: world::DOOR_SLOT + 0.15,
            kind: MugKind::Full,
        },
        Sprite::default(),
        Transform::from_xyz(
            world::slot_x(keg_right, world::DOOR_SLOT + 0.15),
            world::LANE_Y[0] + 7.0,
            5.0,
        ),
        Anchor::CENTER,
        ScreenEntity,
    ));
}

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

fn load_sounds(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SoundAssets {
        pour: assets.load("audio/pour.wav"),
        serve: assets.load("audio/serve.wav"),
        catch: assets.load("audio/catch.wav"),
        empty: assets.load("audio/empty_catch.wav"),
        break_mug: assets.load("audio/break.wav"),
        lane: assets.load("audio/lane.wav"),
        served: assets.load("audio/served.wav"),
        tip: assets.load("audio/tip.wav"),
        grab: assets.load("audio/grab.wav"),
        clear: assets.load("audio/clear.wav"),
        game_over: assets.load("audio/game_over.wav"),
        bonus_win: assets.load("audio/bonus_win.wav"),
        bonus_lose: assets.load("audio/bonus_lose.wav"),
        start: assets.load("audio/start.wav"),
        music: [
            assets.load("audio/music_saloon.wav"),
            assets.load("audio/music_sports.wav"),
            assets.load("audio/music_punk.wav"),
            assets.load("audio/music_alien.wav"),
        ],
    });
}

fn ensure_title(
    mut commands: Commands,
    art: Option<Res<Art>>,
    game: Res<Game>,
    existing: Query<(), With<TitleRoot>>,
) {
    if let Some(art) = art
        && existing.is_empty()
    {
        spawn_title(&mut commands, &art, game.high_score);
    }
}

fn spawn_end_screen(mut commands: Commands, art: Res<Art>, game: Res<Game>) {
    spawn_game_over(&mut commands, &art, &game);
}

fn begin_session(mut game: ResMut<Game>, mut paused: ResMut<Paused>) {
    paused.0 = false;
    if game.fresh {
        game.reset_run();
    } else {
        game.needs_load = true;
    }
}

fn meta_input(
    actions: Res<Actions>,
    state: Res<State<AppState>>,
    mut game: ResMut<Game>,
    mut paused: ResMut<Paused>,
    mut next: ResMut<NextState<AppState>>,
    mut sfx: MessageWriter<Sfx>,
    mut volume: ResMut<GlobalVolume>,
) {
    if actions.mute_pulse {
        game.muted = !game.muted;
        volume.volume = if game.muted {
            Volume::Linear(0.0)
        } else {
            Volume::Linear(0.9)
        };
    }
    if actions.pause_pulse && *state.get() == AppState::InGame {
        paused.0 = !paused.0;
    }

    match state.get() {
        AppState::Title => {
            if actions.start_pulse {
                game.fresh = true;
                paused.0 = false;
                sfx.write(Sfx::Start);
                next.set(AppState::InGame);
            }
        }
        AppState::GameOver => {
            if actions.start_pulse {
                game.fresh = true;
                paused.0 = false;
                sfx.write(Sfx::Start);
                next.set(AppState::InGame);
            } else if actions.pause_pulse {
                next.set(AppState::Title);
            }
        }
        AppState::InGame | AppState::Bonus => {}
    }
}

fn apply_capture(
    capture: Res<CaptureConfig>,
    mut game: ResMut<Game>,
    mut demo: ResMut<Demo>,
    mut next: ResMut<NextState<AppState>>,
) {
    demo.enabled = capture.demo;
    if let Some(screen) = capture.screen {
        game.start_screen = screen.min(world::SCREEN_COUNT - 1);
    }
    if capture.bonus {
        let screen = capture.screen.unwrap_or(4).min(world::SCREEN_COUNT - 1);
        game.setup_screen(screen, 0);
        game.pending = Some(PendingStage {
            next_screen: (screen + 1).min(world::SCREEN_COUNT - 1),
            next_loop: 0,
        });
        next.set(AppState::Bonus);
        return;
    }
    if capture.gameover {
        game.score = 24_750;
        game.high_score = 41_200;
        game.screen = 6;
        game.loop_no = 1;
        game.lives = 0;
        next.set(AppState::GameOver);
        return;
    }
    if capture.start {
        game.fresh = true;
        next.set(AppState::InGame);
    }
}

fn capture_system(
    capture: Res<CaptureConfig>,
    mut frame: Local<u32>,
    mut fired: Local<bool>,
    mut commands: Commands,
) {
    let Some(path) = capture.screenshot.clone() else {
        return;
    };
    if *fired {
        return;
    }
    *frame += 1;
    if *frame < capture.frame.max(1) {
        return;
    }
    *fired = true;
    info!("capturing {} at frame {}", path, *frame);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            move |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

fn parse_args() -> CaptureConfig {
    let mut cfg = CaptureConfig {
        frame: 240,
        ..default()
    };
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--screenshot" => {
                i += 1;
                if i < args.len() {
                    cfg.screenshot = Some(args[i].clone());
                }
            }
            "--frame" => {
                i += 1;
                if i < args.len() {
                    cfg.frame = args[i].parse().unwrap_or(240);
                }
            }
            "--start" => cfg.start = true,
            "--demo" => {
                cfg.demo = true;
                cfg.start = true;
            }
            "--screen" => {
                i += 1;
                if i < args.len() {
                    let n: usize = args[i].parse().unwrap_or(1);
                    cfg.screen = Some(n.saturating_sub(1));
                    cfg.start = true;
                }
            }
            "--bonus" => {
                cfg.bonus = true;
                cfg.start = true;
            }
            "--gameover" => {
                cfg.gameover = true;
            }
            "--mug-break" => {
                cfg.mug_break = true;
                cfg.start = true;
            }
            _ => {}
        }
        i += 1;
    }
    cfg
}

#[allow(dead_code)]
fn _keep_imports() {
    let _ = ScalingMode::FixedVertical {
        viewport_height: 1.0,
    };
    let _ = UiAntiAlias::Off;
}
