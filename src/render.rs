//! Art loading, world spawning and per-frame sprite sync.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::game::{Bartender, Dancer, Game, Mug, MugKind, Patron, PatronState, ScreenEntity, Tip};
use crate::world::*;

#[derive(Resource)]
pub struct Art {
    pub font: Handle<Font>,
    pub logo: Handle<Image>,
    pub backgrounds: [Handle<Image>; THEME_COUNT],
    pub patron_sheets: [Handle<Image>; THEME_COUNT],
    pub patron_layout: Handle<TextureAtlasLayout>,
    pub bartender: Handle<Image>,
    pub bartender_layout: Handle<TextureAtlasLayout>,
    pub kegs: [Handle<Image>; THEME_COUNT],
    pub doors: [Handle<Image>; THEME_COUNT],
    pub mug_full: Handle<Image>,
    pub mug_empty: Handle<Image>,
    pub tip: Handle<Image>,
    pub can: Handle<Image>,
    pub arrow: Handle<Image>,
    pub shaker: Handle<Image>,
    pub dancers: [Handle<Image>; THEME_COUNT],
}

impl Art {
    pub fn background(&self, theme: usize) -> Handle<Image> {
        self.backgrounds[theme.min(THEME_COUNT - 1)].clone()
    }
    pub fn patron_sheet(&self, theme: usize) -> Handle<Image> {
        self.patron_sheets[theme.min(THEME_COUNT - 1)].clone()
    }
    pub fn keg(&self, theme: usize) -> Handle<Image> {
        self.kegs[theme.min(THEME_COUNT - 1)].clone()
    }
    pub fn door(&self, theme: usize) -> Handle<Image> {
        self.doors[theme.min(THEME_COUNT - 1)].clone()
    }
    pub fn dancer(&self, theme: usize) -> Handle<Image> {
        self.dancers[theme.min(THEME_COUNT - 1)].clone()
    }
}

pub fn load_art(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let patron_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(18, 26),
        3,
        4,
        None,
        None,
    ));
    let bartender_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(20, 30),
        6,
        1,
        None,
        None,
    ));
    commands.insert_resource(Art {
        font: assets.load("fonts/Saloon17.ttf"),
        logo: assets.load("sprites/logo.png"),
        backgrounds: [
            assets.load("backgrounds/bg_saloon.png"),
            assets.load("backgrounds/bg_sports.png"),
            assets.load("backgrounds/bg_punk.png"),
            assets.load("backgrounds/bg_alien.png"),
        ],
        patron_sheets: [
            assets.load("sprites/patron_saloon.png"),
            assets.load("sprites/patron_sports.png"),
            assets.load("sprites/patron_punk.png"),
            assets.load("sprites/patron_alien.png"),
        ],
        patron_layout,
        bartender: assets.load("sprites/bartender.png"),
        bartender_layout,
        kegs: [
            assets.load("sprites/keg_saloon.png"),
            assets.load("sprites/keg_sports.png"),
            assets.load("sprites/keg_punk.png"),
            assets.load("sprites/keg_alien.png"),
        ],
        doors: [
            assets.load("sprites/door_saloon.png"),
            assets.load("sprites/door_sports.png"),
            assets.load("sprites/door_punk.png"),
            assets.load("sprites/door_alien.png"),
        ],
        mug_full: assets.load("sprites/mug_full.png"),
        mug_empty: assets.load("sprites/mug_empty.png"),
        tip: assets.load("sprites/tip.png"),
        can: assets.load("sprites/can.png"),
        arrow: assets.load("sprites/arrow.png"),
        shaker: assets.load("sprites/shaker.png"),
        dancers: [
            assets.load("sprites/dancer_saloon.png"),
            assets.load("sprites/dancer_sports.png"),
            assets.load("sprites/dancer_punk.png"),
            assets.load("sprites/dancer_alien.png"),
        ],
    });
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: WORLD_H,
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        bevy::ui_render::prelude::UiAntiAlias::Off,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Main Camera"),
    ));
}

fn sized_sprite(image: Handle<Image>, w: f32, h: f32) -> Sprite {
    Sprite {
        image,
        custom_size: Some(Vec2::new(w, h)),
        ..default()
    }
}

pub fn load_screen(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
    dynamic: Query<Entity, With<ScreenEntity>>,
    old_bg: Query<Entity, With<Backdrop>>,
) {
    if !game.needs_load {
        return;
    }
    game.needs_load = false;

    for e in &dynamic {
        commands.entity(e).despawn();
    }
    for e in &old_bg {
        commands.entity(e).despawn();
    }

    let theme = game.theme();
    commands.spawn((
        sized_sprite(art.background(theme), WORLD_W, WORLD_H),
        Transform::from_xyz(0.0, 0.0, -20.0),
        Backdrop,
        ScreenEntity,
    ));

    for (plan, y) in screen_plan(game.screen).iter().zip(LANE_Y.iter()) {
        let y = *y - 1.0;
        commands.spawn((
            sized_sprite(art.keg(theme), 26.0, 33.0),
            Transform::from_xyz(keg_x(plan.keg_right), y, -5.0),
            Anchor::BOTTOM_CENTER,
            ScreenEntity,
        ));
        commands.spawn((
            sized_sprite(art.door(theme), 24.0, 34.0),
            Transform::from_xyz(door_x(plan.keg_right), y, -8.0),
            Anchor::BOTTOM_CENTER,
            ScreenEntity,
        ));
    }

    commands.spawn((
        Bartender {
            lane: 0,
            slot: BT_KEG_SLOT,
            pour: 0.0,
            facing: 1.0,
            anim: 0.0,
        },
        Sprite {
            image: art.bartender.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: art.bartender_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(slot_x(true, BT_KEG_SLOT), LANE_Y[0] + 1.0, 3.0),
        Anchor::BOTTOM_CENTER,
        ScreenEntity,
    ));
}

#[derive(Component)]
pub struct Backdrop;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn sync_sprites(
    art: Res<Art>,
    game: Res<Game>,
    time: Res<Time>,
    mut commands: Commands,
    mut patrons: Query<(&mut Patron, &mut Transform, &mut Sprite), Without<Mug>>,
    mut mugs: Query<(&mut Mug, &mut Transform, &mut Sprite), Without<Patron>>,
    mut tips: Query<
        (&mut Tip, &mut Transform, &mut Sprite),
        (Without<Patron>, Without<Mug>, Without<Bartender>),
    >,
    mut bartenders: Query<
        (&mut Bartender, &mut Transform, &mut Sprite),
        (Without<Patron>, Without<Mug>, Without<Tip>),
    >,
    mut dancers: Query<
        (Entity, &mut Transform),
        (
            With<Dancer>,
            Without<Patron>,
            Without<Mug>,
            Without<Tip>,
            Without<Bartender>,
        ),
    >,
) {
    let theme = game.theme();
    let plan = screen_plan(game.screen);
    let t = time.elapsed_secs();

    for (patron, mut tf, mut sprite) in &mut patrons {
        let keg_right = plan[patron.lane].keg_right;
        let bob = if patron.distracted {
            (t * 9.0 + patron.lane as f32).sin() * 1.2
        } else {
            0.0
        };
        tf.translation.x = patron_x(keg_right, patron.slot);
        tf.translation.y = LANE_Y[patron.lane] + 1.0 + bob;
        sprite.image = art.patron_sheet(theme);
        sprite.flip_x = !keg_right;
        sprite.texture_atlas = Some(TextureAtlas {
            layout: art.patron_layout.clone(),
            index: patron.variant as usize * 3
                + match patron.state {
                    PatronState::Drinking => 2,
                    _ => (patron.anim * 4.0) as usize % 2,
                },
        });
    }

    for (mug, mut tf, mut sprite) in &mut mugs {
        let keg_right = plan[mug.lane].keg_right;
        tf.translation.x = slot_x(keg_right, mug.slot);
        tf.translation.y = LANE_Y[mug.lane] + 8.0;
        sprite.image = match mug.kind {
            MugKind::Full => art.mug_full.clone(),
            MugKind::Empty => art.mug_empty.clone(),
        };
    }

    for (tip, mut tf, mut sprite) in &mut tips {
        let keg_right = plan[tip.lane].keg_right;
        tf.translation.x = slot_x(keg_right, tip.slot);
        tf.translation.y = LANE_Y[tip.lane] + 9.0 + (t * 5.0 + tip.slot).sin() * 1.0;
        sprite.image = art.tip.clone();
    }

    for (bt, mut tf, mut sprite) in &mut bartenders {
        let keg_right = plan[bt.lane].keg_right;
        tf.translation.x = slot_x(keg_right, bt.slot);
        tf.translation.y = LANE_Y[bt.lane] + 1.0;
        let index = if game.phase == crate::game::Phase::Dying {
            5
        } else if bt.pour > 0.0 {
            3
        } else if (bt.slot - BT_KEG_SLOT).abs() > 0.08 {
            1 + (bt.anim * 7.0) as usize % 2
        } else {
            0
        };
        sprite.texture_atlas = Some(TextureAtlas {
            layout: art.bartender_layout.clone(),
            index,
        });
        sprite.flip_x = keg_right;
    }

    // Dancing distraction: keep three themed dancers on the little stage while
    // the tip timer runs.
    if game.dancers > 0.0 && game.phase == crate::game::Phase::Normal {
        if dancers.is_empty() {
            for i in 0..3 {
                let x = -44.0 + i as f32 * 44.0;
                commands.spawn((
                    Dancer,
                    sized_sprite(art.dancer(theme), 18.0, 26.0),
                    Transform::from_xyz(x, -108.0, -4.0),
                    Anchor::BOTTOM_CENTER,
                    ScreenEntity,
                ));
            }
        }
        for (i, (_, mut tf)) in dancers.iter_mut().enumerate() {
            tf.translation.y = -108.0 + (t * 8.0 + i as f32 * 1.7).sin().abs() * 3.0;
        }
    } else {
        for (e, _) in &dancers {
            commands.entity(e).despawn();
        }
    }
}
