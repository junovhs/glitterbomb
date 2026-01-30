//! WASM particle implementation

use crate::animation::random;
use crate::types::{Color, ConfettiOptions, Shape};
use std::f64::consts::PI;
use web_sys::CanvasRenderingContext2d;

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
    pub fn new(
        opts: &ConfettiOptions,
        start_x: f64,
        start_y: f64,
        color: Color,
        shape: Shape,
    ) -> Self {
        let rad_angle = opts.angle * (PI / 180.0);
        let rad_spread = opts.spread * (PI / 180.0);
        Self {
            x: start_x,
            y: start_y,
            wobble: random() * 10.0,
            wobble_speed: f64::min(0.11, random() * 0.1 + 0.05),
            velocity: (opts.start_velocity * 0.5) + (random() * opts.start_velocity),
            angle_2d: -rad_angle + ((0.5 * rad_spread) - (random() * rad_spread)),
            tilt_angle: (random() * 0.5 + 0.25) * PI,
            color,
            shape,
            tick: 0,
            total_ticks: opts.ticks,
            decay: opts.decay,
            drift: opts.drift,
            random: random() + 2.0,
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

    pub fn update(&mut self) -> bool {
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
            self.random = random() + 2.0;
        }
        self.tick += 1;
        self.tick < self.total_ticks
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d) {
        let progress = f64::from(self.tick) / f64::from(self.total_ticks);
        let x1 = self.x + (self.random * self.tilt_cos);
        let y1 = self.y + (self.random * self.tilt_sin);
        let x2 = self.wobble_x + (self.random * self.tilt_cos);
        let y2 = self.wobble_y + (self.random * self.tilt_sin);

        ctx.set_fill_style_str(&format!(
            "rgba({}, {}, {}, {})",
            self.color.r,
            self.color.g,
            self.color.b,
            1.0 - progress
        ));
        ctx.begin_path();
        match self.shape {
            Shape::Circle => self.render_circle(ctx, x1, x2, y1, y2),
            Shape::Star => self.render_star(ctx),
            Shape::Square => self.render_square(ctx, x1, x2, y1, y2),
        }
        ctx.close_path();
        ctx.fill();
    }

    fn render_circle(&self, ctx: &CanvasRenderingContext2d, x1: f64, x2: f64, y1: f64, y2: f64) {
        let rx = (x2 - x1).abs() * self.oval_scalar;
        let ry = (y2 - y1).abs() * self.oval_scalar;
        ctx.save();
        let _ = ctx.translate(self.x, self.y);
        let _ = ctx.rotate(PI / 10.0 * self.wobble);
        let _ = ctx.scale(rx.max(0.1), ry.max(0.1));
        let _ = ctx.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
        ctx.restore();
    }

    fn render_star(&self, ctx: &CanvasRenderingContext2d) {
        let (inner, outer) = (4.0 * self.scalar, 8.0 * self.scalar);
        let step = PI / 5.0;
        let mut rot = PI / 2.0 * 3.0;
        ctx.move_to(self.x, self.y - outer);
        for _ in 0..5 {
            ctx.line_to(self.x + rot.cos() * outer, self.y + rot.sin() * outer);
            rot += step;
            ctx.line_to(self.x + rot.cos() * inner, self.y + rot.sin() * inner);
            rot += step;
        }
    }

    fn render_square(&self, ctx: &CanvasRenderingContext2d, x1: f64, x2: f64, y1: f64, y2: f64) {
        ctx.move_to(self.x.floor(), self.y.floor());
        ctx.line_to(self.wobble_x.floor(), y1.floor());
        ctx.line_to(x2.floor(), y2.floor());
        ctx.line_to(x1.floor(), self.wobble_y.floor());
    }
}
