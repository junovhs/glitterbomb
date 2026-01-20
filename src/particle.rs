//! Particle state and physics (platform-agnostic).

use crate::renderer::{ConfettiRenderer, Ellipse};
use crate::types::{Color, ConfettiOptions, Shape};
use std::f64::consts::PI;

#[derive(Clone)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    wobble: f64,
    wobble_speed: f64,
    velocity: f64,
    angle_2d: f64,
    tilt_angle: f64,
    color: Color,
    shape: Shape,
    tick: u32,
    total_ticks: u32,
    decay: f64,
    drift: f64,
    random: f64,
    tilt_sin: f64,
    tilt_cos: f64,
    wobble_x: f64,
    wobble_y: f64,
    gravity: f64,
    oval_scalar: f64,
    scalar: f64,
    flat: bool,
}

impl Particle {
    #[must_use]
    pub fn new<F: FnMut() -> f64>(
        opts: &ConfettiOptions,
        start_x: f64,
        start_y: f64,
        color: Color,
        shape: Shape,
        rng: &mut F,
    ) -> Self {
        let rad_angle = opts.angle * (PI / 180.0);
        let rad_spread = opts.spread * (PI / 180.0);

        Self {
            x: start_x,
            y: start_y,
            wobble: rng() * 10.0,
            wobble_speed: f64::min(0.11, rng() * 0.1 + 0.05),
            velocity: (opts.start_velocity * 0.5) + (rng() * opts.start_velocity),
            angle_2d: -rad_angle + ((0.5 * rad_spread) - (rng() * rad_spread)),
            tilt_angle: (rng() * 0.5 + 0.25) * PI,
            color,
            shape,
            tick: 0,
            total_ticks: opts.ticks,
            decay: opts.decay,
            drift: opts.drift,
            random: rng() + 2.0,
            tilt_sin: 0.0,
            tilt_cos: 0.0,
            wobble_x: 0.0,
            wobble_y: 0.0,
            gravity: opts.gravity * 3.0,
            oval_scalar: 0.6,
            scalar: opts.scalar,
            flat: opts.flat,
        }
    }

    /// Update physics. Returns true if still alive.
    pub fn update(&mut self, rng: f64) -> bool {
        self.x += self.angle_2d.cos() * self.velocity + self.drift;
        self.y += self.angle_2d.sin() * self.velocity + self.gravity;
        self.velocity *= self.decay;

        if self.flat {
            self.wobble = 0.0;
            self.wobble_x = self.x + (10.0 * self.scalar);
            self.wobble_y = self.y + (10.0 * self.scalar);
            self.tilt_sin = 0.0;
            self.tilt_cos = 0.0;
            self.random = 1.0;
        } else {
            self.wobble += self.wobble_speed;
            self.wobble_x = self.x + ((10.0 * self.scalar) * self.wobble.cos());
            self.wobble_y = self.y + ((10.0 * self.scalar) * self.wobble.sin());
            self.tilt_angle += 0.1;
            self.tilt_sin = self.tilt_angle.sin();
            self.tilt_cos = self.tilt_angle.cos();
            self.random = rng + 2.0;
        }

        self.tick += 1;
        self.tick < self.total_ticks
    }

    /// Render using any backend.
    pub fn render(&self, renderer: &mut impl ConfettiRenderer) {
        let progress = f64::from(self.tick) / f64::from(self.total_ticks);
        let alpha = 1.0 - progress;

        let x1 = self.x + (self.random * self.tilt_cos);
        let y1 = self.y + (self.random * self.tilt_sin);
        let x2 = self.wobble_x + (self.random * self.tilt_cos);
        let y2 = self.wobble_y + (self.random * self.tilt_sin);

        match self.shape {
            Shape::Circle => {
                renderer.fill_ellipse(&Ellipse {
                    x: self.x,
                    y: self.y,
                    rx: (x2 - x1).abs() * self.oval_scalar,
                    ry: (y2 - y1).abs() * self.oval_scalar,
                    rotation: PI / 10.0 * self.wobble,
                    color: self.color,
                    alpha,
                });
            }
            Shape::Star => {
                let points = self.star_points();
                renderer.fill_polygon(&points, self.color, alpha);
            }
            Shape::Square => {
                let points = [
                    (self.x.floor(), self.y.floor()),
                    (self.wobble_x.floor(), y1.floor()),
                    (x2.floor(), y2.floor()),
                    (x1.floor(), self.wobble_y.floor()),
                ];
                renderer.fill_polygon(&points, self.color, alpha);
            }
        }
    }

    fn star_points(&self) -> Vec<(f64, f64)> {
        let inner = 4.0 * self.scalar;
        let outer = 8.0 * self.scalar;
        let step = PI / 5.0;
        let mut rot = PI / 2.0 * 3.0;
        let mut points = Vec::with_capacity(10);

        for _ in 0..5 {
            points.push((self.x + rot.cos() * outer, self.y + rot.sin() * outer));
            rot += step;
            points.push((self.x + rot.cos() * inner, self.y + rot.sin() * inner));
            rot += step;
        }
        points
    }
}
