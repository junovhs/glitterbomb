//! Platform-agnostic renderer trait.

use crate::types::Color;

/// Parameters for drawing an ellipse.
pub struct Ellipse {
    pub x: f64,
    pub y: f64,
    pub rx: f64,
    pub ry: f64,
    pub rotation: f64,
    pub color: Color,
    pub alpha: f64,
}

/// Trait for rendering confetti to any backend.
pub trait ConfettiRenderer {
    /// Clear the canvas.
    fn clear(&mut self);

    /// Get canvas dimensions.
    fn size(&self) -> (f64, f64);

    /// Draw a filled ellipse.
    fn fill_ellipse(&mut self, ellipse: &Ellipse);

    /// Draw a filled polygon.
    fn fill_polygon(&mut self, points: &[(f64, f64)], color: Color, alpha: f64);

    /// Present the frame.
    fn present(&mut self);
}
