# 💣 Glitterbomb

A pure Rust confetti animation library for WebAssembly. No JavaScript required.

[![Crates.io](https://img.shields.io/crates/v/glitterbomb.svg)](https://crates.io/crates/glitterbomb)
[![Documentation](https://docs.rs/glitterbomb/badge.svg)](https://docs.rs/glitterbomb)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🦀 **Pure Rust** - No JS dependencies, compiles to WebAssembly
- 🎨 **Customizable** - Colors, shapes, physics, and more
- ⚡ **Lightweight** - Minimal dependencies, small WASM binary
- 🎮 **Preset Effects** - Celebration, fireworks, snow, cannon
- ♿ **Accessible** - Respects `prefers-reduced-motion`
- 🔧 **Framework Agnostic** - Works with Dioxus, Leptos, Yew, or vanilla WASM

## Installation

```bash
cargo add glitterbomb
```

## Quick Start

```rust
use glitterbomb::{confetti, ConfettiOptions};

// Fire with defaults
confetti(ConfettiOptions::default());
```

## Usage with Dioxus

```rust
use dioxus::prelude::*;
use glitterbomb::{confetti, celebration, ConfettiOptions, Origin, Color};

#[component]
fn App() -> Element {
    rsx! {
        div {
            // Simple button
            button {
                onclick: move |_| confetti(ConfettiOptions::default()),
                "🎉 Confetti!"
            }

            // Celebration preset (fires from both sides)
            button {
                onclick: move |_| celebration(),
                "🎊 Celebrate!"
            }

            // Custom options
            button {
                onclick: move |_| {
                    confetti(ConfettiOptions {
                        particle_count: 100,
                        spread: 70.0,
                        origin: Origin { x: 0.5, y: 0.8 },
                        colors: vec![
                            Color::from_hex("#ff0000"),
                            Color::from_hex("#00ff00"),
                            Color::from_hex("#0000ff"),
                        ],
                        ..Default::default()
                    });
                },
                "🌈 Custom!"
            }
        }
    }
}
```

## Preset Effects

```rust
use glitterbomb::{celebration, fireworks, snow, cannon};

// Fire from both sides of the screen
celebration();

// Explode from the center like fireworks
fireworks();

// Gentle falling snow effect
snow();

// Blast from the bottom of the screen
cannon();
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `particle_count` | `u32` | `50` | Number of confetti particles |
| `angle` | `f64` | `90.0` | Launch angle in degrees (90 = up) |
| `spread` | `f64` | `45.0` | Spread angle in degrees |
| `start_velocity` | `f64` | `45.0` | Initial particle velocity |
| `decay` | `f64` | `0.9` | Velocity decay rate (0.0 - 1.0) |
| `gravity` | `f64` | `1.0` | Gravity multiplier |
| `drift` | `f64` | `0.0` | Horizontal drift |
| `ticks` | `u32` | `200` | Animation duration (~60 ticks/sec) |
| `origin` | `Origin` | `{x: 0.5, y: 0.5}` | Emission point (0.0 - 1.0) |
| `shapes` | `Vec<Shape>` | `[Square, Circle]` | Particle shapes |
| `colors` | `Vec<Color>` | Rainbow palette | Particle colors |
| `scalar` | `f64` | `1.0` | Size multiplier |
| `z_index` | `i32` | `100` | CSS z-index for canvas |
| `flat` | `bool` | `false` | Disable wobble/rotation |
| `disable_for_reduced_motion` | `bool` | `false` | Respect accessibility setting |

## Shapes

```rust
use glitterbomb::Shape;

let opts = ConfettiOptions {
    shapes: vec![Shape::Square, Shape::Circle, Shape::Star],
    ..Default::default()
};
```

## Colors

```rust
use glitterbomb::Color;

// From hex string
let red = Color::from_hex("#ff0000");
let short = Color::from_hex("#f00"); // Shorthand works too

// From RGB values
let green = Color::new(0, 255, 0);

// Predefined constants
let blue = Color::BLUE;
let white = Color::WHITE;
```

## Custom Canvas

Render confetti on a specific canvas element instead of a fullscreen overlay:

```rust
use glitterbomb::{confetti_on_canvas, ConfettiOptions};
use web_sys::HtmlCanvasElement;

fn fire_on_my_canvas(canvas: &HtmlCanvasElement) {
    confetti_on_canvas(canvas, ConfettiOptions::default());
}
```

## Stop Animation

```rust
use glitterbomb::reset;

// Stop all confetti and remove the canvas
reset();
```

## Accessibility

The library respects the `prefers-reduced-motion` media query when enabled:

```rust
confetti(ConfettiOptions {
    disable_for_reduced_motion: true,
    ..Default::default()
});
```

## Comparison with canvas-confetti

This is a Rust port inspired by [canvas-confetti](https://github.com/catdad/canvas-confetti). Key differences:

| Feature | canvas-confetti | glitterbomb |
|---------|-----------------|----------------|
| Language | JavaScript | Rust/WASM |
| Bundle size | ~15kb min | ~30kb WASM |
| Web Workers | ✅ | ❌ (WASM is fast enough) |
| Custom paths | ✅ | ❌ (coming soon) |
| Text shapes | ✅ | ❌ (coming soon) |
| Bitmap shapes | ✅ | ❌ |

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.
