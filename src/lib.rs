//! # Glitterbomb 💣
//!
//! A pure Rust confetti animation library for WebAssembly and Desktop (wgpu).

#![allow(non_snake_case)]

// Shared types (needed by both web and desktop)
pub mod types;

// Web-only modules
#[cfg(feature = "web")]
mod animation;

// Platform-specific implementations
#[cfg(feature = "desktop")]
pub mod desktop;
#[cfg(feature = "web")]
pub mod web;

// Particle implementations
#[cfg(feature = "web")]
mod particle {
    pub mod wasm;
    pub use wasm::Particle;
}

// Re-exports based on feature
#[cfg(all(feature = "desktop", not(feature = "web")))]
pub use desktop::{cannon, celebration, confetti, confetti_on_canvas, fireworks, reset, snow};
#[cfg(feature = "web")]
pub use web::{cannon, celebration, confetti, confetti_on_canvas, fireworks, reset, snow};
