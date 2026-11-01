//! Tiny particle bursts and camera shake.

use bevy::prelude::*;

use crate::game::Rng;

#[derive(Resource, Default)]
pub struct Fx {
    pub shake: f32,
}

#[derive(Component)]
pub struct Particle {
    vel: Vec2,
    life: f32,
    max_life: f32,
    base: Color,
}

pub fn spawn_burst(commands: &mut Commands, at: Vec2, color: Color, count: usize, speed: f32) {
    for i in 0..count {
        let angle = i as f32 / count.max(1) as f32 * std::f32::consts::TAU;
        let jitter = 0.6 + (i % 3) as f32 * 0.2;
        commands.spawn((
            Particle {
                vel: Vec2::new(angle.cos(), angle.sin()) * speed * jitter,
                life: 0.55,
                max_life: 0.55,
                base: color,
            },
            Sprite::from_color(color, Vec2::splat(2.0)),
            Transform::from_xyz(at.x, at.y, 8.0),
        ));
    }
}

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut p, mut tf, mut sprite) in &mut q {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        p.vel.y -= 120.0 * dt;
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        let t = (p.life / p.max_life).clamp(0.0, 1.0);
        sprite.color = p.base.with_alpha(t);
    }
}

pub fn decay_fx(time: Res<Time>, mut fx: ResMut<Fx>) {
    fx.shake = (fx.shake - time.delta_secs() * 18.0).max(0.0);
}

pub fn camera_shake(
    fx: Res<Fx>,
    mut rng: ResMut<Rng>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut tf) = cam.single_mut() else {
        return;
    };
    if fx.shake > 0.05 {
        tf.translation.x = rng.range(-fx.shake, fx.shake) * 0.45;
        tf.translation.y = rng.range(-fx.shake, fx.shake) * 0.45;
    } else {
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
    }
}
