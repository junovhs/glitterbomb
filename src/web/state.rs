//! Web animation state management.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::WebRenderer;
use crate::{ConfettiRenderer, Particle};

thread_local! {
    pub static STATE: RefCell<Option<AnimState>> = const { RefCell::new(None) };
}

pub struct AnimState {
    pub canvas: HtmlCanvasElement,
    pub renderer: WebRenderer,
    pub particles: Vec<Particle>,
    pub is_animating: bool,
}

impl AnimState {
    pub fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d, w: f64, h: f64) -> Self {
        Self {
            canvas,
            renderer: WebRenderer::new(ctx, w, h),
            particles: Vec::new(),
            is_animating: false,
        }
    }
}

pub fn random() -> f64 {
    js_sys::Math::random()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no window")
}

pub fn prefers_reduced_motion() -> bool {
    window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

pub fn create_canvas(z: i32) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
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
pub fn resize_canvas(c: &HtmlCanvasElement) {
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

pub fn start_loop() {
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
