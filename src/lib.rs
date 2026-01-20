//! # Glitterbomb 💣
//!
//! A pure Rust confetti animation library for WebAssembly.
//! No JavaScript required - just Rust all the way down.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use glitterbomb::{confetti, ConfettiOptions};
//!
//! // Fire with defaults
//! confetti(ConfettiOptions::default());
//!
//! // Or customize it
//! confetti(ConfettiOptions {
//!     particle_count: 100,
//!     spread: 70.0,
//!     origin: Origin { x: 0.5, y: 0.6 },
//!     ..Default::default()
//! });
//! ```
//!
//! ## With Dioxus
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use glitterbomb::{confetti, ConfettiOptions};
//!
//! #[component]
//! fn CelebrationButton() -> Element {
//!     rsx! {
//!         button {
//!             onclick: move |_| {
//!                 confetti(ConfettiOptions::default());
//!             },
//!             "🎉 Celebrate!"
//!         }
//!     }
//! }
//! ```

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// ============================================================================
// PUBLIC TYPES
// ============================================================================

/// RGB color representation
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string like "#ff0000" or "ff0000"
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let hex = if hex.len() == 3 {
            let chars: Vec<char> = hex.chars().collect();
            format!(
                "{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            )
        } else {
            hex.to_string()
        };

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        Self { r, g, b }
    }
}

impl Color {
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255);
}

/// Default confetti color palette
pub fn default_colors() -> Vec<Color> {
    vec![
        Color::from_hex("#26ccff"),
        Color::from_hex("#a25afd"),
        Color::from_hex("#ff5e7e"),
        Color::from_hex("#88ff5a"),
        Color::from_hex("#fcff42"),
        Color::from_hex("#ffa62d"),
        Color::from_hex("#ff36ff"),
    ]
}

/// Shape of confetti particles
#[derive(Clone, Copy, Debug, Default)]
pub enum Shape {
    #[default]
    Square,
    Circle,
    Star,
}

/// Origin point for confetti emission (0.0 to 1.0, relative to canvas)
#[derive(Clone, Copy, Debug)]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

impl Default for Origin {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Configuration options for confetti animation
#[derive(Clone, Debug)]
pub struct ConfettiOptions {
    pub particle_count: u32,
    pub angle: f64,
    pub spread: f64,
    pub start_velocity: f64,
    pub decay: f64,
    pub gravity: f64,
    pub drift: f64,
    pub ticks: u32,
    pub origin: Origin,
    pub shapes: Vec<Shape>,
    pub colors: Vec<Color>,
    pub scalar: f64,
    pub z_index: i32,
    pub flat: bool,
    pub disable_for_reduced_motion: bool,
}

impl Default for ConfettiOptions {
    fn default() -> Self {
        Self {
            particle_count: 50,
            angle: 90.0,
            spread: 45.0,
            start_velocity: 45.0,
            decay: 0.9,
            gravity: 1.0,
            drift: 0.0,
            ticks: 200,
            origin: Origin::default(),
            shapes: vec![Shape::Square, Shape::Circle],
            colors: default_colors(),
            scalar: 1.0,
            z_index: 100,
            flat: false,
            disable_for_reduced_motion: false,
        }
    }
}

// ============================================================================
// INTERNAL PARTICLE STATE
// ============================================================================

#[derive(Clone)]
struct Particle {
    x: f64,
    y: f64,
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
    fn new(opts: &ConfettiOptions, start_x: f64, start_y: f64, color: Color, shape: Shape) -> Self {
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

    fn update(&mut self) -> bool {
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

    fn render(&self, ctx: &CanvasRenderingContext2d) {
        let progress = self.tick as f64 / self.total_ticks as f64;
        let alpha = 1.0 - progress;

        let x1 = self.x + (self.random * self.tilt_cos);
        let y1 = self.y + (self.random * self.tilt_sin);
        let x2 = self.wobble_x + (self.random * self.tilt_cos);
        let y2 = self.wobble_y + (self.random * self.tilt_sin);

        ctx.set_fill_style_str(&format!(
            "rgba({}, {}, {}, {})",
            self.color.r, self.color.g, self.color.b, alpha
        ));

        ctx.begin_path();

        match self.shape {
            Shape::Circle => {
                let radius_x = (x2 - x1).abs() * self.oval_scalar;
                let radius_y = (y2 - y1).abs() * self.oval_scalar;
                let rotation = PI / 10.0 * self.wobble;

                ctx.save();
                let _ = ctx.translate(self.x, self.y);
                let _ = ctx.rotate(rotation);
                let _ = ctx.scale(radius_x.max(0.1), radius_y.max(0.1));
                let _ = ctx.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
                ctx.restore();
            }
            Shape::Star => {
                let inner_radius = 4.0 * self.scalar;
                let outer_radius = 8.0 * self.scalar;
                let spikes = 5;
                let step = PI / spikes as f64;
                let mut rot = PI / 2.0 * 3.0;

                ctx.move_to(self.x, self.y - outer_radius);

                for _ in 0..spikes {
                    let x = self.x + rot.cos() * outer_radius;
                    let y = self.y + rot.sin() * outer_radius;
                    ctx.line_to(x, y);
                    rot += step;

                    let x = self.x + rot.cos() * inner_radius;
                    let y = self.y + rot.sin() * inner_radius;
                    ctx.line_to(x, y);
                    rot += step;
                }
            }
            Shape::Square => {
                ctx.move_to(self.x.floor(), self.y.floor());
                ctx.line_to(self.wobble_x.floor(), y1.floor());
                ctx.line_to(x2.floor(), y2.floor());
                ctx.line_to(x1.floor(), self.wobble_y.floor());
            }
        }

        ctx.close_path();
        ctx.fill();
    }
}

// ============================================================================
// ANIMATION STATE - Use try_borrow_mut to avoid panics
// ============================================================================

thread_local! {
    static ANIMATION_STATE: RefCell<Option<AnimationState>> = const { RefCell::new(None) };
}

struct AnimationState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    particles: Vec<Particle>,
    is_animating: bool,
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Fire confetti with the given options.
pub fn confetti(opts: ConfettiOptions) {
    if opts.disable_for_reduced_motion && prefers_reduced_motion() {
        return;
    }

    let needs_new_animation = ANIMATION_STATE.with(|state| {
        // Use try_borrow_mut to avoid panic if already borrowed
        let Ok(mut state) = state.try_borrow_mut() else {
            return false; // Animation loop is running, just skip
        };

        if state.is_none() {
            let (canvas, ctx) = create_canvas(opts.z_index);
            *state = Some(AnimationState {
                canvas,
                ctx,
                particles: Vec::new(),
                is_animating: false,
            });
        }

        let s = state.as_mut().unwrap();
        resize_canvas(&s.canvas);

        let width = s.canvas.width() as f64;
        let height = s.canvas.height() as f64;
        let start_x = width * opts.origin.x;
        let start_y = height * opts.origin.y;

        for i in 0..opts.particle_count {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = opts.shapes[random_int(0, opts.shapes.len())];
            s.particles.push(Particle::new(&opts, start_x, start_y, color, shape));
        }

        if !s.is_animating {
            s.is_animating = true;
            true
        } else {
            false
        }
    });

    if needs_new_animation {
        start_animation();
    }
}

/// Fire confetti on a specific canvas element.
pub fn confetti_on_canvas(canvas: &HtmlCanvasElement, opts: ConfettiOptions) {
    if opts.disable_for_reduced_motion && prefers_reduced_motion() {
        return;
    }

    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .expect("Could not get 2d context")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Could not cast to CanvasRenderingContext2d");

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let start_x = width * opts.origin.x;
    let start_y = height * opts.origin.y;

    let particles: Vec<Particle> = (0..opts.particle_count)
        .map(|i| {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = opts.shapes[random_int(0, opts.shapes.len())];
            Particle::new(&opts, start_x, start_y, color, shape)
        })
        .collect();

    run_standalone_animation(canvas.clone(), ctx, particles);
}

/// Reset/stop all confetti animations and remove the canvas.
pub fn reset() {
    ANIMATION_STATE.with(|state| {
        if let Ok(mut state) = state.try_borrow_mut() {
            if let Some(s) = state.take() {
                s.canvas.remove();
            }
        }
    });
}

// ============================================================================
// PRESET EFFECTS
// ============================================================================

/// Fire confetti from both sides of the screen.
pub fn celebration() {
    confetti(ConfettiOptions {
        particle_count: 50,
        angle: 60.0,
        spread: 55.0,
        origin: Origin { x: 0.0, y: 0.6 },
        ..Default::default()
    });
    confetti(ConfettiOptions {
        particle_count: 50,
        angle: 120.0,
        spread: 55.0,
        origin: Origin { x: 1.0, y: 0.6 },
        ..Default::default()
    });
}

/// Fire confetti straight up like fireworks.
pub fn fireworks() {
    confetti(ConfettiOptions {
        particle_count: 100,
        spread: 360.0,
        start_velocity: 30.0,
        gravity: 0.5,
        origin: Origin { x: 0.5, y: 0.5 },
        ..Default::default()
    });
}

/// Gentle snow-like falling confetti.
pub fn snow() {
    confetti(ConfettiOptions {
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

/// Confetti cannon from the bottom of the screen.
pub fn cannon() {
    confetti(ConfettiOptions {
        particle_count: 150,
        spread: 60.0,
        start_velocity: 55.0,
        origin: Origin { x: 0.5, y: 1.0 },
        ..Default::default()
    });
}

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

fn window() -> web_sys::Window {
    web_sys::window().expect("no global window")
}

fn document() -> web_sys::Document {
    window().document().expect("no document")
}

fn random() -> f64 {
    js_sys::Math::random()
}

fn random_int(min: usize, max: usize) -> usize {
    (random() * (max - min) as f64).floor() as usize + min
}

fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

fn create_canvas(z_index: i32) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let document = document();
    let canvas = document
        .create_element("canvas")
        .expect("Could not create canvas")
        .dyn_into::<HtmlCanvasElement>()
        .expect("Could not cast to HtmlCanvasElement");

    let style = canvas.style();
    let _ = style.set_property("position", "fixed");
    let _ = style.set_property("top", "0");
    let _ = style.set_property("left", "0");
    let _ = style.set_property("width", "100%");
    let _ = style.set_property("height", "100%");
    let _ = style.set_property("pointer-events", "none");
    let _ = style.set_property("z-index", &z_index.to_string());

    document
        .body()
        .expect("no body")
        .append_child(&canvas)
        .ok();

    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .expect("Could not get 2d context")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Could not cast to CanvasRenderingContext2d");

    (canvas, ctx)
}

fn resize_canvas(canvas: &HtmlCanvasElement) {
    let window = window();
    let width = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0) as u32;
    let height = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0) as u32;

    if canvas.width() != width {
        canvas.set_width(width);
    }
    if canvas.height() != height {
        canvas.set_height(height);
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register RAF")
}

fn start_animation() {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        let should_continue = ANIMATION_STATE.with(|state| {
            let Ok(mut state) = state.try_borrow_mut() else {
                return true; // Can't borrow, try again next frame
            };

            let Some(ref mut s) = *state else {
                return false;
            };

            let width = s.canvas.width() as f64;
            let height = s.canvas.height() as f64;
            s.ctx.clear_rect(0.0, 0.0, width, height);

            s.particles.retain_mut(|p| {
                let alive = p.update();
                if alive {
                    p.render(&s.ctx);
                }
                alive
            });

            !s.particles.is_empty()
        });

        if should_continue {
            request_animation_frame(f.borrow().as_ref().unwrap());
        } else {
            ANIMATION_STATE.with(|state| {
                if let Ok(mut state) = state.try_borrow_mut() {
                    if let Some(s) = state.take() {
                        s.canvas.remove();
                    }
                }
            });
        }
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn run_standalone_animation(
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    particles: Vec<Particle>,
) {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let canvas = Rc::new(canvas);
    let ctx = Rc::new(ctx);
    let particles = Rc::new(RefCell::new(particles));

    let canvas_clone = canvas.clone();
    let ctx_clone = ctx.clone();
    let particles_clone = particles.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        let width = canvas_clone.width() as f64;
        let height = canvas_clone.height() as f64;
        ctx_clone.clear_rect(0.0, 0.0, width, height);

        let mut parts = particles_clone.borrow_mut();
        parts.retain_mut(|p| {
            let alive = p.update();
            if alive {
                p.render(&ctx_clone);
            }
            alive
        });

        if !parts.is_empty() {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

// ============================================================================
// WASM BINDGEN EXPORTS
// ============================================================================

#[wasm_bindgen(js_name = confetti)]
pub fn confetti_js() {
    confetti(ConfettiOptions::default());
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
