//! Contains the 2D grid container.

use crate::cell::{Cell, Material};

/// A 2D grid of simulation cells, stored in row-major order.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Creates a new grid filled entirely with Air cells.
    pub fn new(width: usize, height: usize) -> Grid {
        let cells = (0..width * height)
            .map(|_| Cell::default_for(Material::Air))
            .collect();

        Grid { width, height, cells }
    }

    // -- Accessors --------------------------------------------------------

    /// Returns a reference to the cell at (x, y).
    /// Panics if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.width + x]
    }

    /// Returns a mutable reference to the cell at (x, y).
    /// Panics if out of bounds.
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }

    /// Replaces the cell at (x, y) with the given cell.
    /// Panics if out of bounds.
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * self.width + x] = cell;
    }

    /// Returns true if (x, y) is within the grid bounds.
    /// Accepts i32 so callers can test offsets that might be negative.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    // -- Neighbors --------------------------------------------------------

    /// Returns the Von Neumann neighbors (N, S, E, W) that are in bounds.
    /// Each entry is (nx, ny, &Cell).
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(i32, i32, &Cell)> {
        let offsets: [(i32, i32); 4] = [
            (0, -1), // North
            (0, 1),  // South
            (1, 0),  // East
            (-1, 0), // West
        ];

        let mut result = Vec::with_capacity(4);

        for (dx, dy) in offsets {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if self.in_bounds(nx, ny) {
                result.push((nx, ny, self.get(nx as usize, ny as usize)));
            }
        }

        result
    }

    // -- Fill helpers -----------------------------------------------------

    /// Fills a rectangular region with `Cell::default_for(material)`.
    /// Coordinates are clamped to the grid bounds automatically.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, material: Material) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for cy in y..y_end {
            for cx in x..x_end {
                self.cells[cy * self.width + cx] = Cell::default_for(material.clone());
            }
        }
    }

    /// Draws only the border of a circle (1 cell thick) using the midpoint
    /// circle algorithm. Used to build dome outlines.
    pub fn fill_circle_border(
        &mut self,
        cx: i32,
        cy: i32,
        radius: i32,
        material: Material,
    ) {
        // Midpoint circle algorithm — plots all 8 octant symmetry points.
        let mut x = radius;
        let mut y = 0;
        let mut d = 1 - radius; // decision parameter

        while x >= y {
            // Plot one point in each of the 8 symmetric octants.
            let points: [(i32, i32); 8] = [
                (cx + x, cy + y),
                (cx - x, cy + y),
                (cx + x, cy - y),
                (cx - x, cy - y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx + y, cy - x),
                (cx - y, cy - x),
            ];

            for (px, py) in points {
                if self.in_bounds(px, py) {
                    let ux = px as usize;
                    let uy = py as usize;
                    self.cells[uy * self.width + ux] = Cell::default_for(material.clone());
                }
            }

            y += 1;

            if d <= 0 {
                // Midpoint is inside the circle — move east.
                d += 2 * y + 1;
            } else {
                // Midpoint is outside — move south-east.
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    }
}
