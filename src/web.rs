//! Web (WASM) renderer using Canvas 2D.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{Color, ConfettiOptions, ConfettiRenderer, Origin, Particle};

/// Web canvas renderer.
pub struct WebRenderer {
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
}

impl WebRenderer {
    fn new(ctx: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        Self { ctx, width, height }
    }
}

impl ConfettiRenderer for WebRenderer {
    fn clear(&mut self) {
        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
    }

    fn size(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    fn fill_ellipse(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        color: Color,
        alpha: f64,
    ) {
        self.ctx.set_fill_style_str(&format!(
            "rgba({},{},{},{alpha})",
            color.r, color.g, color.b
        ));
        self.ctx.begin_path();
        self.ctx.save();
        let _ = self.ctx.translate(x, y);
        let _ = self.ctx.rotate(rotation);
        let _ = self.ctx.scale(rx.max(0.1), ry.max(0.1));
        let _ = self.ctx.arc(0.0, 0.0, 1.0, 0.0, std::f64::consts::PI * 2.0);
        self.ctx.restore();
        self.ctx.fill();
    }

    fn fill_polygon(&mut self, points: &[(f64, f64)], color: Color, alpha: f64) {
        if points.is_empty() {
            return;
        }
        self.ctx.set_fill_style_str(&format!(
            "rgba({},{},{},{alpha})",
            color.r, color.g, color.b
        ));
        self.ctx.begin_path();
        self.ctx.move_to(points[0].0, points[0].1);
        for p in &points[1..] {
            self.ctx.line_to(p.0, p.1);
        }
        self.ctx.close_path();
        self.ctx.fill();
    }

    fn present(&mut self) {
        // Canvas auto-presents
    }
}

// ---------------------------------------------------------------------------
// Animation state
// ---------------------------------------------------------------------------

thread_local! {
    static STATE: RefCell<Option<AnimState>> = const { RefCell::new(None) };
}

struct AnimState {
    canvas: HtmlCanvasElement,
    renderer: WebRenderer,
    particles: Vec<Particle>,
    is_animating: bool,
}

fn random() -> f64 {
    js_sys::Math::random()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no window")
}

fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

/// Fire confetti with given options.
///
/// # Panics
/// Panics if DOM is unavailable.
pub fn confetti(opts: &ConfettiOptions) {
    if opts.disable_for_reduced_motion && prefers_reduced_motion() {
        return;
    }

    let needs_start = STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return false;
        };

        if state.is_none() {
            let (canvas, ctx) = create_canvas(opts.z_index);
            let w = f64::from(canvas.width());
            let h = f64::from(canvas.height());
            *state = Some(AnimState {
                canvas,
                renderer: WebRenderer::new(ctx, w, h),
                particles: Vec::new(),
                is_animating: false,
            });
        }

        let s = state.as_mut().expect("just set");
        resize_canvas(&s.canvas);
        s.renderer.width = f64::from(s.canvas.width());
        s.renderer.height = f64::from(s.canvas.height());

        let (w, h) = s.renderer.size();
        let sx = w * opts.origin.x;
        let sy = h * opts.origin.y;

        for i in 0..opts.particle_count {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape =
                opts.shapes[(random() * opts.shapes.len() as f64) as usize % opts.shapes.len()];
            s.particles
                .push(Particle::new(opts, sx, sy, color, shape, random()));
        }

        if s.is_animating {
            false
        } else {
            s.is_animating = true;
            true
        }
    });

    if needs_start {
        start_loop();
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

// Presets
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
pub fn cannon() {
    confetti(&ConfettiOptions {
        particle_count: 150,
        spread: 60.0,
        start_velocity: 55.0,
        origin: Origin { x: 0.5, y: 1.0 },
        ..Default::default()
    });
}

// Internal
fn create_canvas(z: i32) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let doc = window().document().expect("no doc");
    let canvas: HtmlCanvasElement = doc
        .create_element("canvas")
        .expect("create")
        .dyn_into()
        .expect("cast");
    let st = canvas.style();
    let _ = st.set_property("position", "fixed");
    let _ = st.set_property("inset", "0");
    let _ = st.set_property("width", "100%");
    let _ = st.set_property("height", "100%");
    let _ = st.set_property("pointer-events", "none");
    let _ = st.set_property("z-index", &z.to_string());
    doc.body().expect("body").append_child(&canvas).ok();
    resize_canvas(&canvas);
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .expect("ctx")
        .dyn_into()
        .expect("cast");
    (canvas, ctx)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn resize_canvas(c: &HtmlCanvasElement) {
    let w = window();
    let width = w
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0) as u32;
    let height = w
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0) as u32;
    if c.width() != width {
        c.set_width(width);
    }
    if c.height() != height {
        c.set_height(height);
    }
}

fn start_loop() {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        let cont = STATE.with(|state| {
            let Ok(mut state) = state.try_borrow_mut() else {
                return true;
            };
            let Some(ref mut s) = *state else {
                return false;
            };

            s.renderer.clear();
            s.particles.retain_mut(|p| {
                let alive = p.update(random());
                if alive {
                    p.render(&mut s.renderer);
                }
                alive
            });
            !s.particles.is_empty()
        });

        if cont {
            window()
                .request_animation_frame(
                    f.borrow()
                        .as_ref()
                        .expect("closure")
                        .as_ref()
                        .unchecked_ref(),
                )
                .ok();
        } else {
            STATE.with(|s| {
                if let Ok(mut s) = s.try_borrow_mut() {
                    if let Some(st) = s.take() {
                        st.canvas.remove();
                    }
                }
            });
        }
    }));

    window()
        .request_animation_frame(
            g.borrow()
                .as_ref()
                .expect("closure")
                .as_ref()
                .unchecked_ref(),
        )
        .ok();
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
