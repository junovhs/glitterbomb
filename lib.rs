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
            // Expand shorthand like "f00" -> "ff0000"
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

/// Predefined color palettes
impl Color {
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255);
}

/// Default confetti color palette (matches canvas-confetti)
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
    /// Horizontal position (0.0 = left, 1.0 = right)
    pub x: f64,
    /// Vertical position (0.0 = top, 1.0 = bottom)
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
    /// Number of confetti particles to launch
    pub particle_count: u32,
    /// Launch angle in degrees (90 = straight up)
    pub angle: f64,
    /// Spread angle in degrees
    pub spread: f64,
    /// Initial velocity of particles
    pub start_velocity: f64,
    /// Velocity decay rate (0.0 to 1.0)
    pub decay: f64,
    /// Gravity pull (1.0 = normal)
    pub gravity: f64,
    /// Horizontal drift
    pub drift: f64,
    /// Animation duration in ticks (~60 ticks/second)
    pub ticks: u32,
    /// Origin point for particle emission
    pub origin: Origin,
    /// Available shapes for particles
    pub shapes: Vec<Shape>,
    /// Available colors for particles
    pub colors: Vec<Color>,
    /// Size scalar for particles
    pub scalar: f64,
    /// CSS z-index for the canvas
    pub z_index: i32,
    /// If true, particles don't wobble/rotate
    pub flat: bool,
    /// Disable animation if user prefers reduced motion
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

    /// Update particle physics. Returns true if particle is still alive.
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

    /// Render the particle to the canvas
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

                // Draw ellipse manually (wider browser support)
                ctx.save();
                ctx.translate(self.x, self.y).ok();
                ctx.rotate(rotation).ok();
                ctx.scale(radius_x.max(0.1), radius_y.max(0.1)).ok();
                ctx.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI).ok();
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
// ANIMATION STATE
// ============================================================================

thread_local! {
    static ANIMATION_STATE: RefCell<Option<AnimationState>> = const { RefCell::new(None) };
}

struct AnimationState {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    particles: Vec<Particle>,
    animation_id: Option<i32>,
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Fire confetti with the given options.
///
/// Creates a fullscreen canvas overlay and animates confetti particles.
/// The canvas is automatically removed when the animation completes.
///
/// # Example
///
/// ```rust,ignore
/// use glitterbomb::{confetti, ConfettiOptions, Origin};
///
/// // Fire from the bottom center
/// confetti(ConfettiOptions {
///     origin: Origin { x: 0.5, y: 1.0 },
///     particle_count: 100,
///     spread: 70.0,
///     ..Default::default()
/// });
/// ```
pub fn confetti(opts: ConfettiOptions) {
    // Check reduced motion preference
    if opts.disable_for_reduced_motion && prefers_reduced_motion() {
        return;
    }

    ANIMATION_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Create canvas if needed
        let (canvas, ctx) = if let Some(ref mut s) = *state {
            // Reuse existing canvas
            (s.canvas.clone(), s.ctx.clone())
        } else {
            // Create new canvas
            let (canvas, ctx) = create_canvas(opts.z_index);
            *state = Some(AnimationState {
                canvas: canvas.clone(),
                ctx: ctx.clone(),
                particles: Vec::new(),
                animation_id: None,
            });
            (canvas, ctx)
        };

        // Ensure canvas is sized correctly
        resize_canvas(&canvas);

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        // Spawn new particles
        let start_x = width * opts.origin.x;
        let start_y = height * opts.origin.y;

        let state_ref = state.as_mut().unwrap();

        for i in 0..opts.particle_count {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = opts.shapes[random_int(0, opts.shapes.len())];
            state_ref
                .particles
                .push(Particle::new(&opts, start_x, start_y, color, shape));
        }

        // Start animation if not already running
        if state_ref.animation_id.is_none() {
            start_animation();
        }
    });
}

/// Fire confetti using a specific canvas element.
///
/// Useful when you want to render confetti on a specific canvas
/// rather than a fullscreen overlay.
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

    // Run standalone animation for this canvas
    run_standalone_animation(canvas.clone(), ctx, particles);
}

/// Reset/stop all confetti animations and remove the canvas.
pub fn reset() {
    ANIMATION_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(s) = state.take() {
            if let Some(id) = s.animation_id {
                cancel_animation_frame(id);
            }
            s.canvas.remove();
        }
    });
}

// ============================================================================
// PRESET EFFECTS
// ============================================================================

/// Fire confetti from both sides of the screen (like a celebration).
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
    style.set_property("position", "fixed").ok();
    style.set_property("top", "0").ok();
    style.set_property("left", "0").ok();
    style.set_property("width", "100%").ok();
    style.set_property("height", "100%").ok();
    style.set_property("pointer-events", "none").ok();
    style.set_property("z-index", &z_index.to_string()).ok();

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

fn cancel_animation_frame(id: i32) {
    window().cancel_animation_frame(id).ok();
}

fn start_animation() {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        let should_continue = ANIMATION_STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(ref mut s) = *state {
                // Clear canvas
                let width = s.canvas.width() as f64;
                let height = s.canvas.height() as f64;
                s.ctx.clear_rect(0.0, 0.0, width, height);

                // Update and render particles
                s.particles.retain_mut(|p| {
                    let alive = p.update();
                    if alive {
                        p.render(&s.ctx);
                    }
                    alive
                });

                !s.particles.is_empty()
            } else {
                false
            }
        });

        if should_continue {
            let id = request_animation_frame(f.borrow().as_ref().unwrap());
            ANIMATION_STATE.with(|state| {
                if let Some(ref mut s) = *state.borrow_mut() {
                    s.animation_id = Some(id);
                }
            });
        } else {
            // Animation complete, clean up
            ANIMATION_STATE.with(|state| {
                if let Some(s) = state.borrow_mut().take() {
                    s.canvas.remove();
                }
            });
        }
    }));

    let id = request_animation_frame(g.borrow().as_ref().unwrap());
    ANIMATION_STATE.with(|state| {
        if let Some(ref mut s) = *state.borrow_mut() {
            s.animation_id = Some(id);
        }
    });
}

fn run_standalone_animation(
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    mut particles: Vec<Particle>,
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
// WASM BINDGEN EXPORTS (for non-Rust usage)
// ============================================================================

/// Fire confetti with default options (exported to JS)
#[wasm_bindgen(js_name = confetti)]
pub fn confetti_js() {
    confetti(ConfettiOptions::default());
}

/// Fire celebration preset (exported to JS)
#[wasm_bindgen(js_name = celebration)]
pub fn celebration_js() {
    celebration();
}

/// Fire fireworks preset (exported to JS)
#[wasm_bindgen(js_name = fireworks)]
pub fn fireworks_js() {
    fireworks();
}

/// Fire cannon preset (exported to JS)
#[wasm_bindgen(js_name = cannon)]
pub fn cannon_js() {
    cannon();
}

/// Reset all animations (exported to JS)
#[wasm_bindgen(js_name = reset)]
pub fn reset_js() {
    reset();
}
