//! Contains the 2D grid container.

use crate::cell::{Cell, Material};
use crate::rules;
use crate::scheduler::Scheduler;

/// A 2D grid of simulation cells, stored in row-major order.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    // simulation tick
    
    /// Advances the simulation by one tick for the highest-priority cells.
    ///
    /// Uses a two-buffer strategy: all reads happen against the current
    /// `self.cells` snapshot, changes accumulate in a temporary vec, and
    /// writes are applied only after every cell in the batch has been
    /// evaluated.  This prevents update-order artifacts where a cell would
    /// see a half-updated neighbor within the same tick.
    pub fn tick(&mut self, scheduler: &mut Scheduler, max_cells_per_tick: usize) {
        let batch = scheduler.drain_batch(max_cells_per_tick);

        // Accumulated writes: (flat_index, updated_cell).
        let mut writes: Vec<(usize, Cell)> = Vec::with_capacity(batch.len());

        // Neighbor indices to re-enqueue after the write phase.
        // Each entry: (flat_index of neighbor).
        let mut neighbor_enqueues: Vec<usize> = Vec::new();

        // read phase
        for &idx in &batch {
            let x = idx % self.width;
            let y = idx / self.width;

            let cell = &self.cells[idx];
            let neighbors = self.neighbors(x, y);

            // Compute diffusion.
            let new_temp = rules::diffuse_temperature(cell, &neighbors);
            let new_pressure = rules::diffuse_pressure(cell, &neighbors);
            let phase = rules::check_phase_transition(cell);

            // Build the updated cell (clone current, patch fields).
            let mut updated = cell.clone();
            updated.temperature = new_temp;
            updated.pressure = new_pressure;

            // Phase transition.
            if let Some(new_material) = phase {
                rules::apply_phase_transition(&mut updated, new_material);
            }

            // Fire decay.
            if matches!(updated.material, Material::Fire) {
                rules::decay_fire(&mut updated);
            }

            writes.push((idx, updated));

            // Collect neighbor indices for re-enqueue (skip Earth — inert).
            for &(nx, ny, neighbor) in &neighbors {
                if !matches!(neighbor.material, Material::Earth) {
                    let nidx = ny as usize * self.width + nx as usize;
                    neighbor_enqueues.push(nidx);
                }
            }
        }

        // write phase
        for (idx, updated) in &writes {
            self.cells[*idx] = updated.clone();
        }

        // re-enqueue
        // Re-enqueue the cells we just updated (they may still be active).
        for (idx, updated) in &writes {
            scheduler.enqueue_if_active(*idx, updated);
        }

        // Enqueue affected neighbors so they react next tick.
        // We force-enqueue rather than using enqueue_if_active because the
        // neighbor's own state may still look ambient — it's the *adjacent*
        // cell that changed and will drive a non-zero diffusion delta.
        for nidx in neighbor_enqueues {
            scheduler.enqueue(nidx, 1.0);
        }
    }

    /// Creates a new grid filled entirely with Air cells.
    pub fn new(width: usize, height: usize) -> Grid {
        let cells = (0..width * height)
            .map(|_| Cell::default_for(Material::Air))
            .collect();

        Grid { width, height, cells }
    }

    // accessors

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

    // neighbors

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

    // fill helpers

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
        // Midpoint circle algorithm -> plots all 8 octant symmetry points.
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
                // Midpoint is inside the circle -> move east.
                d += 2 * y + 1;
            } else {
                // Midpoint is outside -> move south-east.
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    }
}
