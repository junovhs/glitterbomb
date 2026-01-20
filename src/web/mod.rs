//! Web (WASM) renderer using Canvas 2D.

mod state;

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::renderer::Ellipse;
use crate::{Color, ConfettiOptions, ConfettiRenderer, Origin, Particle};
use state::{create_canvas, prefers_reduced_motion, random, resize_canvas, STATE};

/// Web canvas renderer.
pub struct WebRenderer {
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
}

impl WebRenderer {
    #[must_use]
    pub fn new(ctx: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        Self { ctx, width, height }
    }

    pub fn set_size(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }
}

impl ConfettiRenderer for WebRenderer {
    fn clear(&mut self) {
        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
    }

    fn size(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    fn fill_ellipse(&mut self, e: &Ellipse) {
        self.ctx.set_fill_style_str(&format!(
            "rgba({},{},{},{})",
            e.color.r, e.color.g, e.color.b, e.alpha
        ));
        self.ctx.begin_path();
        self.ctx.save();
        let _ = self.ctx.translate(e.x, e.y);
        let _ = self.ctx.rotate(e.rotation);
        let _ = self.ctx.scale(e.rx.max(0.1), e.ry.max(0.1));
        let _ = self.ctx.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI * 2.0);
        self.ctx.restore();
        self.ctx.fill();
    }

    fn fill_polygon(&mut self, points: &[(f64, f64)], color: Color, alpha: f64) {
        if points.is_empty() {
            return;
        }
        self.ctx.set_fill_style_str(&format!(
            "rgba({},{},{},{})",
            color.r, color.g, color.b, alpha
        ));
        self.ctx.begin_path();
        self.ctx.move_to(points[0].0, points[0].1);
        for p in &points[1..] {
            self.ctx.line_to(p.0, p.1);
        }
        self.ctx.close_path();
        self.ctx.fill();
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

/// Fire confetti with given options.
///
/// # Panics
/// Panics if DOM is unavailable.
pub fn confetti(opts: &ConfettiOptions) {
    if opts.disable_for_reduced_motion && prefers_reduced_motion() {
        return;
    }

    let needs_start = STATE.with(|st| {
        let Ok(mut st) = st.try_borrow_mut() else {
            return false;
        };

        if st.is_none() {
            let (canvas, ctx) = create_canvas(opts.z_index);
            let w = f64::from(canvas.width());
            let h = f64::from(canvas.height());
            *st = Some(state::AnimState::new(canvas, ctx, w, h));
        }

        let s = st.as_mut().expect("just set");
        resize_canvas(&s.canvas);
        s.renderer
            .set_size(f64::from(s.canvas.width()), f64::from(s.canvas.height()));

        let (w, h) = s.renderer.size();
        let sx = w * opts.origin.x;
        let sy = h * opts.origin.y;

        for i in 0..opts.particle_count {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = random_shape(&opts.shapes);
            s.particles
                .push(Particle::new(opts, sx, sy, color, shape, &mut random));
        }

        if s.is_animating {
            false
        } else {
            s.is_animating = true;
            true
        }
    });

    if needs_start {
        state::start_loop();
    }
}

/// Stop and clear.
pub fn reset() {
    STATE.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            if let Some(st) = s.take() {
                st.canvas.remove();
            }
        }
    });
}

/// Fire confetti from both sides.
pub fn celebration() {
    confetti(&ConfettiOptions {
        particle_count: 50,
        angle: 60.0,
        spread: 55.0,
        origin: Origin { x: 0.0, y: 0.6 },
        ..Default::default()
    });
    confetti(&ConfettiOptions {
        particle_count: 50,
        angle: 120.0,
        spread: 55.0,
        origin: Origin { x: 1.0, y: 0.6 },
        ..Default::default()
    });
}

/// Fireworks effect.
pub fn fireworks() {
    confetti(&ConfettiOptions {
        particle_count: 100,
        spread: 360.0,
        start_velocity: 30.0,
        gravity: 0.5,
        origin: Origin { x: 0.5, y: 0.5 },
        ..Default::default()
    });
}

/// Snow effect.
pub fn snow() {
    confetti(&ConfettiOptions {
        particle_count: 50,
        spread: 180.0,
        start_velocity: 10.0,
        gravity: 0.3,
        ticks: 400,
        origin: Origin { x: 0.5, y: 0.0 },
        colors: vec![Color::WHITE, Color::from_hex("#e0e0e0")],
        ..Default::default()
    });
}

/// Cannon effect.
pub fn cannon() {
    confetti(&ConfettiOptions {
        particle_count: 150,
        spread: 60.0,
        start_velocity: 55.0,
        origin: Origin { x: 0.5, y: 1.0 },
        ..Default::default()
    });
}

// WASM exports
#[wasm_bindgen(js_name = confetti)]
pub fn confetti_js() {
    confetti(&ConfettiOptions::default());
}
#[wasm_bindgen(js_name = celebration)]
pub fn celebration_js() {
    celebration();
}
#[wasm_bindgen(js_name = fireworks)]
pub fn fireworks_js() {
    fireworks();
}
#[wasm_bindgen(js_name = cannon)]
pub fn cannon_js() {
    cannon();
}
#[wasm_bindgen(js_name = reset)]
pub fn reset_js() {
    reset();
}
