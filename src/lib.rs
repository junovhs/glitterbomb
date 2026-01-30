//! # Glitterbomb 💣
//!
//! A pure Rust confetti animation library for WebAssembly and Desktop (wgpu).
//! No JavaScript required.

#![allow(non_snake_case)]

// Platform-specific modules
#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "desktop")]
pub mod desktop;

// Shared modules (always available, but web-specific for now)
#[cfg(feature = "web")]
mod animation;

#[cfg(feature = "web")]
mod types;

#[cfg(feature = "web")]
mod particle {
    pub mod wasm;
    pub use wasm::Particle;
}

// Re-exports
#[cfg(feature = "web")]
pub use types::{default_colors, Color, ConfettiOptions, Origin, Shape};

#[cfg(feature = "web")]
pub use web::{cannon, celebration, confetti, confetti_on_canvas, fireworks, reset, snow};

#[cfg(feature = "desktop")]
pub use desktop::{cannon, celebration, confetti, confetti_on_canvas, fireworks, reset, snow};
