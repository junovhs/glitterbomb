//! Public types for confetti configuration.

/// RGB color representation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    pub const CYAN: Self = Self::new(0, 255, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let hex = if hex.len() == 3 {
            let chars: Vec<char> = hex.chars().collect();
            format!(
                "{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            )
        } else {
            hex.to_string()
        };

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        Self { r, g, b }
    }
}

/// Default confetti color palette
#[must_use]
pub fn default_colors() -> Vec<Color> {
    vec![
        Color::from_hex("#26ccff"),
        Color::from_hex("#a25afd"),
        Color::from_hex("#ff5e7e"),
        Color::from_hex("#88ff5a"),
        Color::from_hex("#fcff42"),
        Color::from_hex("#ffa62d"),
        Color::from_hex("#ff36ff"),
    ]
}

/// Shape of confetti particles
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shape {
    #[default]
    Square,
    Circle,
    Star,
}

/// Origin point for confetti emission (0.0 to 1.0, relative to canvas)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

impl Default for Origin {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Configuration options for confetti animation
#[derive(Clone, Debug)]
pub struct ConfettiOptions {
    pub particle_count: u32,
    pub angle: f64,
    pub spread: f64,
    pub start_velocity: f64,
    pub decay: f64,
    pub gravity: f64,
    pub drift: f64,
    pub ticks: u32,
    pub origin: Origin,
    pub shapes: Vec<Shape>,
    pub colors: Vec<Color>,
    pub scalar: f64,
    pub z_index: i32,
    pub flat: bool,
    pub disable_for_reduced_motion: bool,
}

impl Default for ConfettiOptions {
    fn default() -> Self {
        Self {
            particle_count: 50,
            angle: 90.0,
            spread: 45.0,
            start_velocity: 45.0,
            decay: 0.9,
            gravity: 1.0,
            drift: 0.0,
            ticks: 200,
            origin: Origin::default(),
            shapes: vec![Shape::Square, Shape::Circle],
            colors: default_colors(),
            scalar: 1.0,
            z_index: 100,
            flat: false,
            disable_for_reduced_motion: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_hex_full() {
        assert_eq!(Color::from_hex("#ff0000"), Color::new(255, 0, 0));
        assert_eq!(Color::from_hex("#00ff00"), Color::new(0, 255, 0));
        assert_eq!(Color::from_hex("#0000ff"), Color::new(0, 0, 255));
        assert_eq!(Color::from_hex("#a5b7c9"), Color::new(0xa5, 0xb7, 0xc9));
    }

    #[test]
    fn color_from_hex_short() {
        assert_eq!(Color::from_hex("#f00"), Color::new(255, 0, 0));
        assert_eq!(Color::from_hex("#0f0"), Color::new(0, 255, 0));
        assert_eq!(Color::from_hex("#abc"), Color::new(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn color_from_hex_no_hash() {
        assert_eq!(Color::from_hex("ff0000"), Color::new(255, 0, 0));
        assert_eq!(Color::from_hex("f00"), Color::new(255, 0, 0));
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::RED, Color::new(255, 0, 0));
        assert_eq!(Color::GREEN, Color::new(0, 255, 0));
        assert_eq!(Color::BLUE, Color::new(0, 0, 255));
        assert_eq!(Color::WHITE, Color::new(255, 255, 255));
    }

    #[test]
    fn default_colors_not_empty() {
        let colors = default_colors();
        assert_eq!(colors.len(), 7);
        assert_eq!(colors[0], Color::new(0x26, 0xcc, 0xff));
    }

    #[test]
    fn origin_default() {
        let o = Origin::default();
        assert!((o.x - 0.5).abs() < f64::EPSILON);
        assert!((o.y - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn confetti_options_defaults() {
        let opts = ConfettiOptions::default();
        assert_eq!(opts.particle_count, 50);
        assert!((opts.angle - 90.0).abs() < f64::EPSILON);
        assert!((opts.spread - 45.0).abs() < f64::EPSILON);
        assert!((opts.decay - 0.9).abs() < f64::EPSILON);
        assert_eq!(opts.ticks, 200);
        assert!(!opts.shapes.is_empty());
        assert!(!opts.colors.is_empty());
    }

    #[test]
    fn shape_default() {
        assert_eq!(Shape::default(), Shape::Square);
    }
}
