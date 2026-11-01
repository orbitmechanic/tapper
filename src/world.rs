//! Layout, level tables, difficulty curves and pure spatial helpers.

pub const WORLD_W: f32 = 320.0;
pub const WORLD_H: f32 = 240.0;

pub const LANE_COUNT: usize = 4;
pub const SCREEN_COUNT: usize = 13;
pub const THEME_COUNT: usize = 4;

/// World-space walking surface for each bar, top lane first.
pub const LANE_Y: [f32; LANE_COUNT] = [54.0, 8.0, -38.0, -84.0];

/// Centre of the keg end of every bar, mirrored per lane.
pub const KEG_X: f32 = 134.0;
pub const DOOR_X: f32 = 134.0;
/// Distance between adjacent patron slots.
pub const SLOT_SPACING: f32 = 27.0;
/// Patrons spawn this many slots out from the keg (just inside the door).
pub const SPAWN_SLOT: f32 = 8.65;
/// A patron pushed to this slot leaves through the door and is scored.
pub const DOOR_SLOT: f32 = 9.45;
/// A patron that reaches this slot grabs the bartender.
pub const GRAB_SLOT: f32 = 0.52;
/// The bartender's resting position relative to the keg.
pub const BT_KEG_SLOT: f32 = 0.62;
/// Furthest the bartender can run from the keg.
pub const BT_MAX_SLOT: f32 = 8.2;
/// Where a fresh full mug starts sliding.
pub const POUR_SLOT: f32 = 0.72;

pub const BAR_HALF: f32 = 140.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanePlan {
    /// Number of patrons served by this lane on the screen.
    pub count: u8,
    /// True when the keg is on the right and patrons enter from the left.
    pub keg_right: bool,
}

impl LanePlan {
    pub const fn new(count: u8, keg_right: bool) -> Self {
        Self { count, keg_right }
    }

    pub fn direction(self) -> f32 {
        if self.keg_right { 1.0 } else { -1.0 }
    }
}

pub const SCREENS: [[LanePlan; LANE_COUNT]; SCREEN_COUNT] = [
    // 1-2: saloon
    [
        LanePlan::new(1, true),
        LanePlan::new(1, true),
        LanePlan::new(1, true),
        LanePlan::new(1, true),
    ],
    [
        LanePlan::new(2, true),
        LanePlan::new(2, true),
        LanePlan::new(2, true),
        LanePlan::new(2, true),
    ],
    // 3-5: sports bar
    [
        LanePlan::new(2, true),
        LanePlan::new(3, true),
        LanePlan::new(3, true),
        LanePlan::new(3, true),
    ],
    [
        LanePlan::new(2, true),
        LanePlan::new(3, true),
        LanePlan::new(3, true),
        LanePlan::new(4, true),
    ],
    [
        LanePlan::new(3, true),
        LanePlan::new(3, true),
        LanePlan::new(3, true),
        LanePlan::new(4, true),
    ],
    // 6-9: punk bar, lanes start to mirror
    [
        LanePlan::new(2, false),
        LanePlan::new(3, false),
        LanePlan::new(3, true),
        LanePlan::new(4, true),
    ],
    [
        LanePlan::new(3, true),
        LanePlan::new(3, true),
        LanePlan::new(4, false),
        LanePlan::new(4, false),
    ],
    [
        LanePlan::new(3, false),
        LanePlan::new(4, false),
        LanePlan::new(4, true),
        LanePlan::new(4, true),
    ],
    [
        LanePlan::new(4, true),
        LanePlan::new(4, true),
        LanePlan::new(4, false),
        LanePlan::new(4, false),
    ],
    // 10-13: alien bar, fully alternating
    [
        LanePlan::new(4, false),
        LanePlan::new(4, true),
        LanePlan::new(4, true),
        LanePlan::new(4, false),
    ],
    [
        LanePlan::new(4, true),
        LanePlan::new(4, false),
        LanePlan::new(4, false),
        LanePlan::new(4, true),
    ],
    [
        LanePlan::new(4, false),
        LanePlan::new(4, true),
        LanePlan::new(4, false),
        LanePlan::new(4, true),
    ],
    [
        LanePlan::new(4, true),
        LanePlan::new(4, false),
        LanePlan::new(4, true),
        LanePlan::new(4, false),
    ],
];

pub const THEME_NAMES: [&str; THEME_COUNT] = ["SALOON", "SPORTS BAR", "PUNK BAR", "ALIEN BAR"];
pub const THEME_SUBS: [&str; THEME_COUNT] = [
    "THE DUSTY SPUR",
    "THE HOME TEAM",
    "THE PIT",
    "THE LAST ROUND",
];
pub const PATRON_SCORES: [u32; THEME_COUNT] = [50, 75, 100, 150];
pub const SCREEN_CLEAR_BONUS: u32 = 1000;
pub const EMPTY_MUG_BONUS: u32 = 100;
pub const TIP_BONUS: u32 = 1500;
pub const BONUS_WIN: u32 = 3000;

pub fn theme_for_screen(screen: usize) -> usize {
    match screen {
        0..=1 => 0,
        2..=4 => 1,
        5..=8 => 2,
        _ => 3,
    }
}

pub fn screen_plan(screen: usize) -> [LanePlan; LANE_COUNT] {
    SCREENS[screen.min(SCREEN_COUNT - 1)]
}

/// True when a shell-game bonus round follows the given zero-based screen.
pub fn milestone_after(screen: usize) -> bool {
    matches!(screen, 1 | 4 | 8 | 12)
}

/// Zero-based index of the screen that follows `screen`, plus the loop delta.
pub fn next_screen(screen: usize) -> (usize, u32) {
    if screen + 1 >= SCREEN_COUNT {
        (0, 1)
    } else {
        (screen + 1, 0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Difficulty {
    pub spawn_interval: f32,
    pub advance_speed: f32,
    pub push_slots: f32,
    pub drink_time: f32,
    pub max_active: usize,
    pub serve_speed: f32,
    pub empty_speed: f32,
}

pub fn difficulty(screen: usize, loop_no: u32) -> Difficulty {
    let s = screen as f32;
    let l = loop_no as f32;
    Difficulty {
        spawn_interval: (3.55 - s * 0.125 - l * 0.28).max(1.15),
        advance_speed: 0.125 + s * 0.0105 + l * 0.045,
        push_slots: (2.55 - s * 0.052 - l * 0.16).max(1.05),
        drink_time: (0.92 - s * 0.014 - l * 0.03).max(0.55),
        max_active: (1 + screen / 4 + loop_no as usize / 2).min(4),
        serve_speed: 3.35,
        empty_speed: 2.05 + s * 0.012,
    }
}

/// Signed speed multiplier for horizontal movement along a bar.
pub fn along_bar_sign(keg_right: bool) -> f32 {
    if keg_right { -1.0 } else { 1.0 }
}

pub fn slot_x(keg_right: bool, slot: f32) -> f32 {
    if keg_right {
        KEG_X - slot * SLOT_SPACING
    } else {
        -KEG_X + slot * SLOT_SPACING
    }
}

pub fn keg_x(keg_right: bool) -> f32 {
    if keg_right { KEG_X } else { -KEG_X }
}

pub fn door_x(keg_right: bool) -> f32 {
    if keg_right { -DOOR_X } else { DOOR_X }
}

/// X coordinate of a sprite advancing with `dir` where +1 is toward the door.
pub fn patron_x(keg_right: bool, slot: f32) -> f32 {
    slot_x(keg_right, slot)
}

/// A bartender can pluck an empty mug while standing near the keg.
pub fn can_catch_empty(bartender_slot: f32) -> bool {
    bartender_slot <= BT_KEG_SLOT + 0.9
}

/// Whether a patron pushed by `push` leaves through the door.
pub fn pushed_out(slot: f32, push: f32) -> bool {
    slot + push >= DOOR_SLOT
}

pub fn score_for(theme: usize) -> u32 {
    // Every patron in a theme is worth the same; the theme sets the value.
    PATRON_SCORES[theme.min(THEME_COUNT - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirteen_screens_repeat_the_four_themes() {
        let themes: Vec<usize> = (0..SCREEN_COUNT).map(theme_for_screen).collect();
        assert_eq!(themes, vec![0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    }

    #[test]
    fn bonus_follows_each_theme() {
        let milestones: Vec<usize> = (0..SCREEN_COUNT).filter(|s| milestone_after(*s)).collect();
        assert_eq!(milestones, vec![1, 4, 8, 12]);
    }

    #[test]
    fn after_screen_thirteen_the_loop_advances() {
        assert_eq!(next_screen(0), (1, 0));
        assert_eq!(next_screen(12), (0, 1));
    }

    #[test]
    fn screen_layouts_match_the_arcade_pattern() {
        assert_eq!(screen_plan(0)[0].count, 1);
        assert_eq!(screen_plan(1)[0].count, 2);
        assert_eq!(screen_plan(3).map(|l| l.count), [2, 3, 3, 4]);
        assert_eq!(
            screen_plan(5).map(|l| l.keg_right),
            [false, false, true, true]
        );
        assert_eq!(screen_plan(9).map(|l| l.count), [4, 4, 4, 4]);
        assert_eq!(
            screen_plan(12).map(|l| l.keg_right),
            [true, false, true, false]
        );
    }

    #[test]
    fn slots_are_mirrored_around_the_bar() {
        assert_eq!(slot_x(true, 0.0), KEG_X);
        assert_eq!(slot_x(false, 0.0), -KEG_X);
        assert!(slot_x(true, 8.65) < 0.0);
        assert!(slot_x(false, 8.65) > 0.0);
        assert_eq!(door_x(true), -DOOR_X);
        assert_eq!(door_x(false), DOOR_X);
    }

    #[test]
    fn difficulty_rises_and_stays_in_bounds() {
        let first = difficulty(0, 0);
        let last = difficulty(12, 0);
        let looped = difficulty(12, 4);
        assert!(last.spawn_interval < first.spawn_interval);
        assert!(last.advance_speed > first.advance_speed);
        assert!(last.push_slots < first.push_slots);
        assert!(looped.advance_speed > last.advance_speed);
        assert!(looped.spawn_interval >= 1.15);
        assert!(looped.push_slots >= 1.05);
        assert!(looped.max_active <= 4);
    }

    #[test]
    fn pushback_and_grab_windows() {
        assert!(!pushed_out(8.0, 1.0));
        assert!(pushed_out(8.5, 1.0));
        assert!(can_catch_empty(BT_KEG_SLOT));
        assert!(!can_catch_empty(4.0));
    }
}
