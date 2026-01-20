//! Desktop renderer using tiny-skia + minifb.

use minifb::{Key, Window, WindowOptions};
use std::cell::Cell;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::renderer::Ellipse;
use crate::{Color, ConfettiOptions, ConfettiRenderer, Particle};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

thread_local! {
    static RNG_SEED: Cell<u64> = Cell::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345)
    );
}

fn random() -> f64 {
    RNG_SEED.with(|seed| {
        let s = seed
            .get()
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed.set(s);
        (s >> 33) as f64 / f64::from(1u32 << 31)
    })
}

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
                ((p.alpha() as u32) << 24)
                    | ((p.red() as u32) << 16)
                    | ((p.green() as u32) << 8)
                    | (p.blue() as u32)
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

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn fill_ellipse(&mut self, e: &Ellipse) {
        let mut paint = Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(
            e.color.r,
            e.color.g,
            e.color.b,
            (e.alpha * 255.0) as u8,
        ));
        paint.anti_alias = true;

        let rect = tiny_skia::Rect::from_xywh(
            (e.x - e.rx) as f32,
            (e.y - e.ry) as f32,
            (e.rx * 2.0) as f32,
            (e.ry * 2.0) as f32,
        );

        if let Some(rect) = rect {
            let mut pb = PathBuilder::new();
            pb.push_oval(rect);
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
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

    fn present(&mut self) {}
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn random_shape(shapes: &[crate::Shape]) -> crate::Shape {
    shapes[(random() * shapes.len() as f64) as usize % shapes.len()]
}

/// Fire confetti in a new window.
pub fn confetti(opts: &ConfettiOptions) {
    run_window(opts);
}

/// Open a window and run confetti animation.
#[allow(clippy::cast_possible_truncation)]
pub fn run_window(opts: &ConfettiOptions) {
    let mut window =
        Window::new("Glitterbomb", WIDTH, HEIGHT, WindowOptions::default()).expect("create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

    let mut renderer = DesktopRenderer::new(WIDTH as u32, HEIGHT as u32);
    let (w, h) = renderer.size();
    let sx = w * opts.origin.x;
    let sy = h * opts.origin.y;

    let mut particles: Vec<Particle> = (0..opts.particle_count)
        .map(|i| {
            let color = opts.colors[i as usize % opts.colors.len()];
            Particle::new(opts, sx, sy, color, random_shape(&opts.shapes), &mut random)
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
        window
            .update_with_buffer(&renderer.buffer_argb(), WIDTH, HEIGHT)
            .expect("update");
    }
}

/// Celebration preset.
pub fn celebration() {
    run_window(&ConfettiOptions {
        particle_count: 100,
        spread: 70.0,
        ..Default::default()
    });
}

/// Fireworks preset.
pub fn fireworks() {
    run_window(&ConfettiOptions {
        particle_count: 100,
        spread: 360.0,
        start_velocity: 30.0,
        gravity: 0.5,
        ..Default::default()
    });
}
