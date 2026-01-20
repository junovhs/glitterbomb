//! Platform-agnostic renderer trait.

use crate::types::Color;

/// Trait for rendering confetti to any backend.
pub trait ConfettiRenderer {
    /// Clear the canvas.
    fn clear(&mut self);

    /// Get canvas dimensions.
    fn size(&self) -> (f64, f64);

    /// Draw a filled ellipse.
    fn fill_ellipse(
        &mut self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        color: Color,
        alpha: f64,
    );

    /// Draw a filled polygon.
    fn fill_polygon(&mut self, points: &[(f64, f64)], color: Color, alpha: f64);

    /// Present the frame (
    fn present(&mut self);
}
