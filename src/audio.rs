//! Sound effects and per-theme music.

use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;

use crate::AppState;
use crate::game::Game;

#[derive(Message, Clone, Copy, Debug)]
pub enum Sfx {
    Pour,
    Serve,
    Catch,
    EmptyCatch,
    Break,
    Lane,
    Served,
    Tip,
    Grab,
    LevelClear,
    GameOver,
    BonusWin,
    BonusLose,
    Start,
    Extra,
}

#[derive(Resource)]
pub struct SoundAssets {
    pub pour: Handle<AudioSource>,
    pub serve: Handle<AudioSource>,
    pub catch: Handle<AudioSource>,
    pub empty: Handle<AudioSource>,
    pub break_mug: Handle<AudioSource>,
    pub lane: Handle<AudioSource>,
    pub served: Handle<AudioSource>,
    pub tip: Handle<AudioSource>,
    pub grab: Handle<AudioSource>,
    pub clear: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub bonus_win: Handle<AudioSource>,
    pub bonus_lose: Handle<AudioSource>,
    pub start: Handle<AudioSource>,
    pub music: [Handle<AudioSource>; 4],
}

impl SoundAssets {
    pub fn get(&self, sfx: Sfx) -> Handle<AudioSource> {
        match sfx {
            Sfx::Pour => self.pour.clone(),
            Sfx::Serve => self.serve.clone(),
            Sfx::Catch => self.catch.clone(),
            Sfx::EmptyCatch => self.empty.clone(),
            Sfx::Break => self.break_mug.clone(),
            Sfx::Lane => self.lane.clone(),
            Sfx::Served => self.served.clone(),
            Sfx::Tip => self.tip.clone(),
            Sfx::Grab => self.grab.clone(),
            Sfx::LevelClear => self.clear.clone(),
            Sfx::GameOver => self.game_over.clone(),
            Sfx::BonusWin => self.bonus_win.clone(),
            Sfx::BonusLose => self.bonus_lose.clone(),
            Sfx::Start => self.start.clone(),
            Sfx::Extra => self.clear.clone(),
        }
    }
}

fn volume_for(sfx: Sfx) -> f32 {
    match sfx {
        Sfx::Pour => 0.30,
        Sfx::Serve => 0.34,
        Sfx::Catch => 0.34,
        Sfx::EmptyCatch => 0.42,
        Sfx::Break => 0.55,
        Sfx::Lane => 0.20,
        Sfx::Served => 0.40,
        Sfx::Tip => 0.52,
        Sfx::Grab => 0.60,
        Sfx::LevelClear => 0.55,
        Sfx::GameOver => 0.60,
        Sfx::BonusWin => 0.55,
        Sfx::BonusLose => 0.48,
        Sfx::Start => 0.45,
        Sfx::Extra => 0.50,
    }
}

pub fn play_sfx(
    mut commands: Commands,
    mut messages: MessageReader<Sfx>,
    sounds: Res<SoundAssets>,
) {
    for sfx in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.get(*sfx)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume_for(*sfx))),
        ));
    }
}

#[derive(Component)]
pub struct MusicPlayer;

#[derive(Resource, Default)]
pub struct MusicState {
    pub current: Option<usize>,
    pub entity: Option<Entity>,
}

pub fn update_music(
    mut commands: Commands,
    state: Res<State<AppState>>,
    game: Res<Game>,
    sounds: Res<SoundAssets>,
    mut music: ResMut<MusicState>,
) {
    let desired = match state.get() {
        AppState::Title => Some(0),
        AppState::InGame | AppState::Bonus => Some(game.theme()),
        AppState::GameOver => None,
    };
    if desired == music.current {
        return;
    }
    if let Some(entity) = music.entity.take() {
        commands.entity(entity).despawn();
    }
    if let Some(theme) = desired {
        let entity = commands
            .spawn((
                AudioPlayer::new(sounds.music[theme].clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.26)),
                MusicPlayer,
            ))
            .id();
        music.entity = Some(entity);
    }
    music.current = desired;
}

pub fn apply_mute(mut volume: ResMut<GlobalVolume>, game: Res<Game>) {
    volume.volume = if game.muted {
        Volume::Linear(0.0)
    } else {
        Volume::Linear(0.9)
    };
}
