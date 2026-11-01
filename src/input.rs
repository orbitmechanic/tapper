//! Unified keyboard + touch input.

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Pour,
}

const ACTION_COUNT: usize = 5;

#[derive(Resource, Default)]
pub struct PadInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub pour: bool,
    pub up_seq: u64,
    pub down_seq: u64,
    pub left_seq: u64,
    pub right_seq: u64,
    pub pour_seq: u64,
    pub start_seq: u64,
    pub pause_seq: u64,
    pub mute_seq: u64,
    pub touch_mode: bool,
}

#[derive(Resource, Default)]
pub struct Actions {
    held: [bool; ACTION_COUNT],
    prev: [bool; ACTION_COUNT],
    pub start_pulse: bool,
    pub pause_pulse: bool,
    pub mute_pulse: bool,
    pulse: [bool; ACTION_COUNT],
    seen_up: u64,
    seen_down: u64,
    seen_left: u64,
    seen_right: u64,
    seen_pour: u64,
    seen_start: u64,
    seen_pause: u64,
    seen_mute: u64,
}

impl Actions {
    pub fn pressed(&self, action: Action) -> bool {
        self.held[action as usize]
    }

    pub fn just(&self, action: Action) -> bool {
        let i = action as usize;
        (self.held[i] && !self.prev[i]) || self.pulse[i]
    }

    pub fn set(&mut self, action: Action, value: bool) {
        self.held[action as usize] = value;
    }

    pub fn set_start(&mut self) {
        self.start_pulse = true;
    }

    pub fn set_pause(&mut self) {
        self.pause_pulse = true;
    }
}

#[allow(unused_mut, unused_variables)]
pub fn poll_touch(mut touch: ResMut<PadInput>) {
    #[cfg(target_arch = "wasm32")]
    {
        let raw = crate::mobile::read();
        touch.up = raw.up;
        touch.down = raw.down;
        touch.left = raw.left;
        touch.right = raw.right;
        touch.pour = raw.pour;
        touch.up_seq = raw.up_seq;
        touch.down_seq = raw.down_seq;
        touch.left_seq = raw.left_seq;
        touch.right_seq = raw.right_seq;
        touch.pour_seq = raw.pour_seq;
        touch.start_seq = raw.start_seq;
        touch.pause_seq = raw.pause_seq;
        touch.mute_seq = raw.mute_seq;
        touch.touch_mode = raw.touch_mode;
    }
}

pub fn update_actions(
    keys: Res<ButtonInput<KeyCode>>,
    touch: Res<PadInput>,
    mut actions: ResMut<Actions>,
) {
    let held = [
        keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) || touch.up,
        keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) || touch.down,
        keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) || touch.left,
        keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) || touch.right,
        keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyJ) || touch.pour,
    ];

    actions.prev = actions.held;
    actions.held = held;
    actions.pulse = [
        touch.up_seq != actions.seen_up,
        touch.down_seq != actions.seen_down,
        touch.left_seq != actions.seen_left,
        touch.right_seq != actions.seen_right,
        touch.pour_seq != actions.seen_pour,
    ];
    actions.seen_up = touch.up_seq;
    actions.seen_down = touch.down_seq;
    actions.seen_left = touch.left_seq;
    actions.seen_right = touch.right_seq;
    actions.seen_pour = touch.pour_seq;

    let start = keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space)
        || touch.start_seq != actions.seen_start;
    let pause = keys.just_pressed(KeyCode::KeyP)
        || keys.just_pressed(KeyCode::Escape)
        || touch.pause_seq != actions.seen_pause;
    let mute = keys.just_pressed(KeyCode::KeyM) || touch.mute_seq != actions.seen_mute;

    actions.seen_start = touch.start_seq;
    actions.seen_pause = touch.pause_seq;
    actions.seen_mute = touch.mute_seq;
    actions.start_pulse = start;
    actions.pause_pulse = pause;
    actions.mute_pulse = mute;
}

pub fn clear_pulses(mut actions: ResMut<Actions>) {
    actions.start_pulse = false;
    actions.pause_pulse = false;
    actions.mute_pulse = false;
    actions.pulse = [false; ACTION_COUNT];
}
