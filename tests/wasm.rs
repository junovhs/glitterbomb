//! WASM browser tests - Run with: wasm-pack test --headless --firefox

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use glitterbomb::{cannon, celebration, confetti, default_colors, fireworks, reset, snow};
use glitterbomb::{cannon_js, celebration_js, confetti_js, fireworks_js, reset_js};
use glitterbomb::{Color, ConfettiOptions, Origin, Shape};

#[wasm_bindgen_test]
fn presets() {
    celebration(); reset();
    fireworks(); reset();
    snow(); reset();
    cannon(); reset();
}

#[wasm_bindgen_test]
fn js_bindings() {
    confetti_js(); reset_js();
    celebration_js(); reset_js();
    fireworks_js(); reset_js();
    cannon_js(); reset_js();
}

#[wasm_bindgen_test]
fn confetti_and_reset() {
    confetti(&ConfettiOptions::default());
    reset();
    confetti(&ConfettiOptions::default());
    reset();
}

#[wasm_bindgen_test]
fn origin_positions() {
    for origin in [
        Origin { x: 0.0, y: 0.0 },
        Origin { x: 1.0, y: 1.0 },
        Origin { x: 0.5, y: 0.5 },
        Origin { x: 0.25, y: 0.75 },
    ] {
        confetti(&ConfettiOptions { origin, particle_count: 5, ticks: 1, ..Default::default() });
    }
    reset();
}

#[wasm_bindgen_test]
fn color_modulo() {
    confetti(&ConfettiOptions {
        particle_count: 100, colors: vec![Color::RED], ticks: 1, ..Default::default()
    });
    reset();
}

#[wasm_bindgen_test]
fn shape_selection() {
    confetti(&ConfettiOptions {
        particle_count: 50, shapes: vec![Shape::Star], ticks: 1, ..Default::default()
    });
    reset();
}

#[wasm_bindgen_test]
fn physics_params() {
    for opts in [
        ConfettiOptions { gravity: 0.0, ticks: 2, ..Default::default() },
        ConfettiOptions { gravity: 10.0, ticks: 2, ..Default::default() },
        ConfettiOptions { start_velocity: 100.0, decay: 0.5, ticks: 2, ..Default::default() },
        ConfettiOptions { spread: 360.0, angle: 0.0, ticks: 2, ..Default::default() },
        ConfettiOptions { drift: 10.0, ticks: 2, ..Default::default() },
        ConfettiOptions { flat: true, ticks: 2, ..Default::default() },
        ConfettiOptions { scalar: 3.0, ticks: 2, ..Default::default() },
    ] {
        confetti(&opts);
    }
    reset();
}

#[wasm_bindgen_test]
fn all_shapes() {
    for shape in [Shape::Square, Shape::Circle, Shape::Star] {
        confetti(&ConfettiOptions { shapes: vec![shape], particle_count: 20, ticks: 3, ..Default::default() });
    }
    reset();
}

#[wasm_bindgen_test]
fn colors() {
    assert_eq!(default_colors().len(), 7);
    assert_eq!(Color::from_hex("#ff0000"), Color::new(255, 0, 0));
    assert_eq!(Color::from_hex("#f00"), Color::new(255, 0, 0));
}

#[wasm_bindgen_test]
fn edge_cases() {
    confetti(&ConfettiOptions { particle_count: 0, ..Default::default() });
    confetti(&ConfettiOptions { particle_count: 500, ticks: 1, ..Default::default() });
    reset();
}

#[wasm_bindgen_test]
fn reduced_motion() {
    confetti(&ConfettiOptions { disable_for_reduced_motion: true, ticks: 1, ..Default::default() });
    confetti(&ConfettiOptions { disable_for_reduced_motion: false, ticks: 1, ..Default::default() });
    reset();
}
