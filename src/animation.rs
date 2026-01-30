//! Animation state and rendering loop.

use crate::particle::Particle;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

thread_local! {
    pub static ANIMATION_STATE: RefCell<Option<AnimationState>> = const { RefCell::new(None) };
}

pub struct AnimationState {
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub particles: Vec<Particle>,
    pub is_animating: bool,
}

impl AnimationState {
    pub fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Self {
        Self {
            canvas,
            ctx,
            particles: Vec::new(),
            is_animating: false,
        }
    }
}

pub fn random() -> f64 {
    js_sys::Math::random()
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn random_int(min: usize, max: usize) -> usize {
    (random() * (max - min) as f64).floor() as usize + min
}

pub fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global window")
}

fn document() -> web_sys::Document {
    window().document().expect("no document")
}

pub fn get_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .ok()
        .flatten()
        .expect("Could not get 2d context")
        .dyn_into::<CanvasRenderingContext2d>()
        .expect("Could not cast to CanvasRenderingContext2d")
}

pub fn create_canvas(z_index: i32) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
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

    document.body().expect("no body").append_child(&canvas).ok();

    let ctx = get_context(&canvas);
    (canvas, ctx)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn resize_canvas(canvas: &HtmlCanvasElement) {
    let window = window();
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0) as u32;
    let height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0) as u32;

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

pub fn start_animation() {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        let should_continue = ANIMATION_STATE.with(|state| {
            let Ok(mut state) = state.try_borrow_mut() else {
                return true;
            };

            let Some(ref mut s) = *state else {
                return false;
            };

            let width = f64::from(s.canvas.width());
            let height = f64::from(s.canvas.height());
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
            request_animation_frame(f.borrow().as_ref().expect("closure exists"));
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

    request_animation_frame(g.borrow().as_ref().expect("closure exists"));
}

pub fn run_standalone_animation(
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
        let width = f64::from(canvas_clone.width());
        let height = f64::from(canvas_clone.height());
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
            request_animation_frame(f.borrow().as_ref().expect("closure exists"));
        }
    }));

    request_animation_frame(g.borrow().as_ref().expect("closure exists"));
}
