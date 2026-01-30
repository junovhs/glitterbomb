# project -- Semantic Map

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

`slopchop.toml`
Configuration for slopchop. Centralizes project configuration.

## Layer 1 -- Core

`src/lib.rs`
# Glitterbomb 💣  A pure Rust confetti animation library. Provides application entry point.

`src/web/mod.rs`
Web (WASM) renderer using Canvas 2D. Supports application functionality.
→ Exports: WebRenderer, cannon, cannon_js, celebration, celebration_js, confetti, confetti_js, fireworks, fireworks_js, new, reset, reset_js, set_size, snow

## Layer 2 -- Domain

`src/desktop.rs`
Desktop renderer using tiny-skia + minifb. Supports application functionality.
→ Exports: DesktopRenderer, celebration, confetti, fireworks, run_window

`src/particle.rs`
Particle state and physics (platform-agnostic). Supports application functionality.
→ Exports: Particle, new, render, update

`src/renderer.rs`
Platform-agnostic renderer trait. Formats data for output.
→ Exports: ConfettiRenderer, Ellipse

`src/types.rs`
Public types for confetti configuration. Defines domain data structures.
→ Exports: Color, ConfettiOptions, Origin, Shape, default_colors, from_hex

`src/web/state.rs`
Web animation state management. Supports application functionality.
→ Exports: AnimState, create_canvas, new, prefers_reduced_motion, random, resize_canvas, start_loop

## Layer 4 -- Tests

`examples/desktop_test.rs`
Run with: cargo run --example desktop_test --features desktop --no-default-features. Verifies correctness.

