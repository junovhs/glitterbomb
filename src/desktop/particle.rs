//! Desktop particle physics

use crate::types::{Color, ConfettiOptions};
use rand::Rng;

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: [f32; 4],
    pub life: u32,
}

impl Particle {
    pub fn new(opts: &ConfettiOptions, start_x: f32, start_y: f32, color: Color) -> Self {
        let mut rng = rand::thread_rng();
        let rad_angle = (opts.angle as f32).to_radians();
        let rad_spread = (opts.spread as f32).to_radians();
        let angle = rad_angle + (rng.gen::<f32>() - 0.5) * rad_spread;
        let velocity = (opts.start_velocity as f32) * (0.5 + rng.gen::<f32>());

        Self {
            x: start_x,
            y: start_y,
            vx: angle.cos() * velocity,
            vy: -angle.sin() * velocity,
            color: [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                1.0,
            ],
            life: opts.ticks,
        }
    }

    pub fn update(&mut self, gravity: f32, drift: f32) -> bool {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += gravity;
        self.x += drift;
        self.vx *= 0.99;
        self.life -= 1;
        self.life > 0
    }
}
