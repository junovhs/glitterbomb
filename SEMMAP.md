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
# Glitterbomb 💣  A pure Rust confetti animation library for WebAssembly. Provides application entry point.
→ Exports: cannon, cannon_js, celebration, celebration_js, confetti, confetti_js, confetti_on_canvas, fireworks, fireworks_js, reset, reset_js, snow

## Layer 2 -- Domain

`src/animation.rs`
Animation state and rendering loop. Supports application functionality.
→ Exports: AnimationState, create_canvas, get_context, new, prefers_reduced_motion, random, random_int, resize_canvas, run_standalone_animation, start_animation

`src/particle.rs`
Particle state and physics. Supports application functionality.
→ Exports: Particle, TestCfg, new, render, test, update

`src/types.rs`
Public types for confetti configuration. Defines domain data structures.
→ Exports: Color, ConfettiOptions, Origin, Shape, default_colors, from_hex

## Layer 4 -- Tests

`tests/wasm.rs`
WASM browser tests - Run with: wasm-pack test --headless --firefox. Verifies correctness.

