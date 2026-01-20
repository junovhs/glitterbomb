//! # Glitterbomb 💣
//!
//! A pure Rust confetti animation library.
//! Works on web (WASM) and desktop.
//!
//! ## Web
//!
//! ```rust,ignore
//! use glitterbomb::{confetti, ConfettiOptions};
//! confetti(&ConfettiOptions::default());
//! ```
//!
//! ## Desktop
//!
//! ```rust,ignore
//! use glitterbomb::{ConfettiOptions, desktop};
//! desktop::run_window(&ConfettiOptions::default());
//! ```

mod particle;
mod renderer;
mod types;

pub use particle::Particle;
pub use renderer::{ConfettiRenderer, Ellipse};
pub use types::{default_colors, Color, ConfettiOptions, Origin, Shape};

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use web::confetti;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
pub use desktop::confetti;
