//! Public types for confetti configuration.

/// RGB color representation.
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
    pub const BLACK: Self = Self::new(0, 0, 0);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let expanded = if hex.len() == 3 {
            let c: Vec<char> = hex.chars().collect();
            format!("{}{}{}{}{}{}", c[0], c[0], c[1], c[1], c[2], c[2])
        } else {
            hex.to_string()
        };

        Self {
            r: u8::from_str_radix(&expanded[0..2], 16).unwrap_or(0),
            g: u8::from_str_radix(&expanded[2..4], 16).unwrap_or(0),
            b: u8::from_str_radix(&expanded[4..6], 16).unwrap_or(0),
        }
    }

    #[must_use]
    pub const fn to_rgba(self, alpha: u8) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (alpha as u32)
    }

    #[must_use]
    pub const fn to_argb(self, alpha: u8) -> u32 {
        ((alpha as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Default confetti color palette.
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

/// Shape of confetti particles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shape {
    #[default]
    Square,
    Circle,
    Star,
}

/// Origin point for confetti emission (0.0 to 1.0).
#[derive(Clone, Copy, Debug)]
pub struct Origin {
    pub x: f64,
    pub y: f64,
}

impl Default for Origin {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

/// Configuration options for confetti animation.
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
