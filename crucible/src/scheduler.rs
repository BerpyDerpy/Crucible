//! Manages the priority queue for active cells.
//!
//! The scheduler decides which cells get updated each tick. Active cells
//! (those with interesting physics — high temp, pressure, or mana) are
//! enqueued with a priority score. Cold, static cells sleep until something
//! disturbs them.

use std::collections::BinaryHeap;

use ordered_float::OrderedFloat;

use crate::cell::{Cell, Material};
use crate::grid::Grid;

/// Minimum priority score for a cell to be worth updating.
const SLEEP_THRESHOLD: f32 = 0.1;

/// A priority-based scheduler that controls which cells tick each frame.
#[derive(Debug, Clone)]
pub struct Scheduler {
    /// Max-heap of (priority, flat_index).
    queue: BinaryHeap<(OrderedFloat<f32>, usize)>,
    /// Tracks which flat indices are currently in the queue to avoid duplicates.
    in_queue: Vec<bool>,
}

impl Scheduler {
    /// Creates a new scheduler with room for `capacity` cells, all initially
    /// marked as not-in-queue.
    pub fn new(capacity: usize) -> Scheduler {
        Scheduler {
            queue: BinaryHeap::new(),
            in_queue: vec![false; capacity],
        }
    }

    /// Adds a cell (by flat index) to the queue with the given priority.
    /// Does nothing if the cell is already enqueued.
    pub fn enqueue(&mut self, idx: usize, priority: f32) {
        if !self.in_queue[idx] {
            self.in_queue[idx] = true;
            self.queue.push((OrderedFloat(priority), idx));
        }
    }

    /// Computes a priority score for the cell and enqueues it if the score
    /// exceeds `SLEEP_THRESHOLD`.
    ///
    /// The score rewards cells that deviate from ambient conditions or carry
    /// mana, making them candidates for physics updates.
    pub fn enqueue_if_active(&mut self, idx: usize, cell: &Cell) {
        let score = (cell.temperature - 20.0).abs() * 0.1
            + (cell.pressure - 1.0).abs() * 2.0
            + cell.mana_density * 10.0;

        if score > SLEEP_THRESHOLD {
            self.enqueue(idx, score);
        }
    }

    /// Pops up to `max` highest-priority cell indices for this tick.
    /// Each popped cell is marked as not-in-queue so it can be re-enqueued
    /// later if it remains active.
    pub fn drain_batch(&mut self, max: usize) -> Vec<usize> {
        let mut batch = Vec::with_capacity(max);

        for _ in 0..max {
            match self.queue.pop() {
                Some((_priority, idx)) => {
                    self.in_queue[idx] = false;
                    batch.push(idx);
                }
                None => break,
            }
        }

        batch
    }

    /// Scans every cell in the grid and enqueues the active ones.
    /// Skips Earth and Vacuum cells since they are inert at startup.
    /// Call this once when a scenario is first loaded.
    pub fn seed_from_grid(&mut self, grid: &Grid) {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let cell = grid.get(x, y);

                // Earth and Vacuum are inert — skip them.
                match cell.material {
                    Material::Earth | Material::Vacuum => continue,
                    _ => {}
                }

                let idx = y * grid.width + x;
                self.enqueue_if_active(idx, cell);
            }
        }
    }
}
