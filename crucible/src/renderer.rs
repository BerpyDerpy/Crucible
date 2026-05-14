//! Handles pixel framebuffer rendering.
//!
//! Maps simulation grid state to RGBA pixel colors and pushes frames
//! through the `pixels` crate's wgpu-backed surface.

use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

use crate::cell::Material;
use crate::grid::Grid;

// ── Color helpers ───────────────────────────────────────────────────────

/// Linearly interpolate between two u8 color channels.
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

/// Linearly interpolate between two RGB triples.
fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [
        lerp_u8(a[0], b[0], t),
        lerp_u8(a[1], b[1], t),
        lerp_u8(a[2], b[2], t),
    ]
}

/// Blend `base` 30% toward `overlay` (used for the mana debug tint).
fn mana_tint(base: [u8; 3]) -> [u8; 3] {
    let overlay: [u8; 3] = [180, 100, 255];
    lerp_rgb(base, overlay, 0.30)
}

// ── Color mapping ───────────────────────────────────────────────────────

/// Computes the display RGB for a single cell based on its material,
/// temperature, and pressure.
fn cell_color(material: Material, temperature: f32, pressure: f32) -> [u8; 3] {
    match material {
        Material::Vacuum => [0, 0, 0],

        Material::Air => {
            // Cool dark blue → warm sky blue as temp goes 0→200.
            let t = (temperature / 200.0).clamp(0.0, 1.0);
            lerp_rgb([30, 30, 50], [180, 210, 255], t)
        }

        Material::Earth => {
            // Brown → molten orange as temp approaches 1200.
            let t = (temperature / 1200.0).clamp(0.0, 1.0);
            lerp_rgb([80, 50, 20], [200, 60, 0], t)
        }

        Material::Fire => {
            // Deep orange → bright yellow-white as temp goes 200→800.
            let t = ((temperature - 200.0) / 600.0).clamp(0.0, 1.0);
            lerp_rgb([255, 60, 0], [255, 255, 100], t)
        }

        Material::Water => [20, 80, 200],

        Material::Steam => {
            // Gray-blue → white based on pressure (higher = denser cloud).
            // Pressure range roughly 0→20, normalize against ~5 for visual.
            let t = (pressure / 5.0).clamp(0.0, 1.0);
            lerp_rgb([180, 200, 220], [255, 255, 255], t)
        }

        Material::Ice => [180, 220, 255],

        Material::Lava => {
            // Dark red → bright orange as temp goes 200→1200.
            let t = ((temperature - 200.0) / 1000.0).clamp(0.0, 1.0);
            lerp_rgb([200, 20, 0], [255, 180, 0], t)
        }
    }
}

// ── Renderer ────────────────────────────────────────────────────────────

/// Framebuffer renderer that maps grid cells to screen pixels via the
/// `pixels` crate.
pub struct Renderer {
    pixels: Pixels,
    /// Number of screen pixels per simulation grid cell.
    cell_size: usize,
}

impl Renderer {
    /// Creates a new renderer backed by a wgpu surface attached to `window`.
    ///
    /// The framebuffer resolution is `grid_width * cell_size` by
    /// `grid_height * cell_size`.
    pub fn new(
        window: &Window,
        grid_width: usize,
        grid_height: usize,
        cell_size: usize,
    ) -> Self {
        let fb_width = (grid_width * cell_size) as u32;
        let fb_height = (grid_height * cell_size) as u32;

        let window_size = window.inner_size();
        let surface = SurfaceTexture::new(window_size.width, window_size.height, window);

        let pixels = Pixels::new(fb_width, fb_height, surface)
            .expect("failed to initialize pixels framebuffer");

        Renderer { pixels, cell_size }
    }

    /// Rasterizes the entire grid into the framebuffer.
    ///
    /// Each grid cell is drawn as a `cell_size × cell_size` block of
    /// identically-colored pixels.  Cells with mana density above 0.1
    /// receive a soft purple debug tint.
    pub fn draw(&mut self, grid: &Grid) {
        let frame = self.pixels.frame_mut();
        let fb_width = (grid.width * self.cell_size) as usize;

        for gy in 0..grid.height {
            for gx in 0..grid.width {
                let cell = grid.get(gx, gy);

                // Base color from material + physical state.
                let mut rgb = cell_color(cell.material, cell.temperature, cell.pressure);

                // Debug overlay: soft purple tint for mana-dense cells.
                if cell.mana_density > 0.1 {
                    rgb = mana_tint(rgb);
                }

                // Fill the cell_size × cell_size block in the framebuffer.
                let px_x = gx * self.cell_size;
                let px_y = gy * self.cell_size;

                for dy in 0..self.cell_size {
                    for dx in 0..self.cell_size {
                        let offset = ((px_y + dy) * fb_width + (px_x + dx)) * 4;
                        frame[offset] = rgb[0];     // R
                        frame[offset + 1] = rgb[1]; // G
                        frame[offset + 2] = rgb[2]; // B
                        frame[offset + 3] = 255;     // A
                    }
                }
            }
        }

        // ── Coordinate scale overlay ────────────────────────────────────
        // Draw white tick marks every 10 grid cells along the top and left
        // edges of the framebuffer so coordinates are readable at a glance.
        let tick_len = self.cell_size * 2; // tick mark length in pixels
        let tick_color: [u8; 3] = [255, 255, 255];
        let fb_height = grid.height * self.cell_size;

        // X-axis ticks along the top edge.
        for grid_coord in (0..grid.width).step_by(10) {
            let px_x = grid_coord * self.cell_size + self.cell_size / 2;
            for py in 0..tick_len.min(fb_height) {
                let offset = (py * fb_width + px_x) * 4;
                if offset + 3 < frame.len() {
                    frame[offset]     = tick_color[0];
                    frame[offset + 1] = tick_color[1];
                    frame[offset + 2] = tick_color[2];
                    frame[offset + 3] = 255;
                }
            }
        }

        // Y-axis ticks along the left edge.
        for grid_coord in (0..grid.height).step_by(10) {
            let py = grid_coord * self.cell_size + self.cell_size / 2;
            for px_x in 0..tick_len.min(fb_width) {
                let offset = (py * fb_width + px_x) * 4;
                if offset + 3 < frame.len() {
                    frame[offset]     = tick_color[0];
                    frame[offset + 1] = tick_color[1];
                    frame[offset + 2] = tick_color[2];
                    frame[offset + 3] = 255;
                }
            }
        }
    }

    /// Pushes the current framebuffer to the window surface.
    pub fn render(&mut self) {
        self.pixels.render().unwrap();
    }
}
