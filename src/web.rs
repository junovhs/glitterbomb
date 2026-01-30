//! WebAssembly implementation using HTML5 Canvas

use crate::animation;
use crate::particle::Particle;
use crate::types::{Color, ConfettiOptions, Origin};
use animation::{run_standalone_animation, start_animation, AnimationState, ANIMATION_STATE};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// Fire confetti with the given options.
///
/// # Panics
///
/// Panics if the animation state cannot be borrowed (already borrowed elsewhere).
pub fn confetti(opts: &ConfettiOptions) {
    if opts.disable_for_reduced_motion && animation::prefers_reduced_motion() {
        return;
    }

    let needs_new_animation = ANIMATION_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return false;
        };

        if state.is_none() {
            let (canvas, ctx) = animation::create_canvas(opts.z_index);
            *state = Some(AnimationState::new(canvas, ctx));
        }

        let s = state.as_mut().expect("state was just set");
        animation::resize_canvas(&s.canvas);

        let width = f64::from(s.canvas.width());
        let height = f64::from(s.canvas.height());
        let start_x = width * opts.origin.x;
        let start_y = height * opts.origin.y;

        for i in 0..opts.particle_count {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = opts.shapes[animation::random_int(0, opts.shapes.len())];
            s.particles
                .push(Particle::new(opts, start_x, start_y, color, shape));
        }

        if s.is_animating {
            false
        } else {
            s.is_animating = true;
            true
        }
    });

    if needs_new_animation {
        start_animation();
    }
}

/// Fire confetti on a specific canvas element.
///
/// # Panics
///
/// Panics if the canvas 2D context cannot be obtained.
pub fn confetti_on_canvas(canvas: &HtmlCanvasElement, opts: &ConfettiOptions) {
    if opts.disable_for_reduced_motion && animation::prefers_reduced_motion() {
        return;
    }

    let ctx = animation::get_context(canvas);
    let width = f64::from(canvas.width());
    let height = f64::from(canvas.height());
    let start_x = width * opts.origin.x;
    let start_y = height * opts.origin.y;

    let particles: Vec<Particle> = (0..opts.particle_count)
        .map(|i| {
            let color = opts.colors[i as usize % opts.colors.len()];
            let shape = opts.shapes[animation::random_int(0, opts.shapes.len())];
            Particle::new(opts, start_x, start_y, color, shape)
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

/// Fire confetti from both sides of the screen.
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

/// Fire confetti straight up like fireworks.
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

/// Gentle snow-like falling confetti.
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

/// Confetti cannon from the bottom of the screen.
pub fn cannon() {
    confetti(&ConfettiOptions {
        particle_count: 150,
        spread: 60.0,
        start_velocity: 55.0,
        origin: Origin { x: 0.5, y: 1.0 },
        ..Default::default()
    });
}

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
