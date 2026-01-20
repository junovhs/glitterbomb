use glitterbomb::{Color, ConfettiOptions, Origin};

fn main() {
    glitterbomb::desktop::run_window(&ConfettiOptions {
        particle_count: 100,
        spread: 70.0,
        colors: vec![Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW],
        origin: Origin { x: 0.5, y: 0.5 },
        ..Default::default()
    });
}
