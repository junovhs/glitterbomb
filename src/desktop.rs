//! Desktop renderer using tiny-skia + minifb.

use minifb::{Key, Window, WindowOptions};
use std::time::Instant;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::{Color, ConfettiOptions, ConfettiRenderer, Particle};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub struct DesktopRenderer {
    pixmap: Pixmap,
}

impl DesktopRenderer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            pixmap: Pixmap::new(width, height).expect("create pixmap"),
        }
    }

    fn buffer_argb(&self) -> Vec<u32> {
        self.pixmap
            .pixels()
            .iter()
            .map(|p| {
                let r = p.red();
                let g = p.green();
                let b = p.blue();
                let a = p.alpha();
                ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
            })
            .collect()
    }
}

impl ConfettiRenderer for DesktopRenderer {
    fn clear(&mut self) {
        self.pixmap
            .fill(tiny_skia::Color::from_rgba8(240, 240, 240, 255));
    }

    fn size(&self) -> (f64, f64) {
        (
            f64::from(self.pixmap.width()),
            f64::from(self.pixmap.height()),
        )
    }

    fn fill_ellipse(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        _rotation: f64,
        color: Color,
        alpha: f64,
    ) {
        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(
            color.r,
            color.g,
            color.b,
            (alpha * 255.0) as u8,
        ));
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.push_oval(
            tiny_skia::Rect::from_xywh(
                (x - rx) as f32,
                (y - ry) as f32,
                (rx * 2.0) as f32,
                (ry * 2.0) as f32,
            )
            .unwrap_or(tiny_skia::Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap()),
        );

        if let Some(path) = pb.finish() {
            self.pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn fill_polygon(&mut self, points: &[(f64, f64)], color: Color, alpha: f64) {
        if points.is_empty() {
            return;
        }

        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(
            color.r,
            color.g,
            color.b,
            (alpha * 255.0) as u8,
        ));
        paint.anti_alias = true;

        let mut pb = PathBuilder::new();
        pb.move_to(points[0].0 as f32, points[0].1 as f32);
        for p in &points[1..] {
            pb.line_to(p.0 as f32, p.1 as f32);
        }
        pb.close();

        if let Some(path) = pb.finish() {
            self.pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    fn present(&mut self) {
        // minifb handles this in the main loop
    }
}

fn random() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_usize(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as usize)
            .unwrap_or(0),
    );
    (hasher.finish() % 10000) as f64 / 10000.0
}

/// Fire confetti (adds particles to next `run_window` call or running window).
pub fn confetti(opts: &ConfettiOptions) {
    run_window(opts);
}

/// Open a window and run the confetti animation.
pub fn run_window(opts: &ConfettiOptions) {
    let mut window = Window::new("Glitterbomb 💣", WIDTH, HEIGHT, WindowOptions::default())
        .expect("create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16666))); // ~60fps

    let mut renderer = DesktopRenderer::new(WIDTH as u32, HEIGHT as u32);
    let (w, h) = renderer.size();
    let sx = w * opts.origin.x;
    let sy = h * opts.origin.y;

    let mut particles: Vec<Particle> = (0..opts.particle_count)
        .map(|i| {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape =
                opts.shapes[(random() * opts.shapes.len() as f64) as usize % opts.shapes.len()];
            Particle::new(opts, sx, sy, color, shape, random())
        })
        .collect();

    while window.is_open() && !window.is_key_down(Key::Escape) && !particles.is_empty() {
        renderer.clear();

        particles.retain_mut(|p| {
            let alive = p.update(random());
            if alive {
                p.render(&mut renderer);
            }
            alive
        });

        let buf = renderer.buffer_argb();
        window
            .update_with_buffer(&buf, WIDTH, HEIGHT)
            .expect("update");
    }
}

// Presets
pub fn celebration() {
    // Desktop opens one window, so we just run with celebration-like settings
    run_window(&ConfettiOptions {
        particle_count: 100,
        spread: 70.0,
        start_velocity: 50.0,
        ..Default::default()
    });
}

pub fn fireworks() {
    run_window(&ConfettiOptions {
        particle_count: 100,
        spread: 360.0,
        start_velocity: 30.0,
        gravity: 0.5,
        ..Default::default()
    });
}
