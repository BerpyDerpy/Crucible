//! Provides hardcoded test scenarios.
//!
//! Each scenario function sets up a specific grid configuration for
//! testing or demonstration purposes.

use crate::cell::Material;
use crate::grid::Grid;
use crate::scheduler::Scheduler;

/// Builds the dome-test scenario: an earth dome enclosing fire and water.
///
/// Layout on a 120×120 grid:
/// - Earth circle border at center (60, 60), radius 35
/// - 3×3 fire block at (59..62, 61..64) — heat source near dome floor
/// - 3×3 water block at (59..62, 39..42) — suspended above fire
/// - All non-Earth, non-Vacuum cells marked active
pub fn dome_test(grid: &mut Grid) {
    // 1. Fill entire grid with Air (Grid::new already does this, but be
    //    explicit in case the grid was reused).
    grid.fill_rect(0, 0, grid.width, grid.height, Material::Air);

    // 2. Draw the earth dome border.
    grid.fill_circle_border(60, 60, 25, Material::Earth);

    // 3. Place the fire source (3×3 block centered at (60, 62)).
    grid.fill_rect(59, 61, 3, 3, Material::Fire);

    // 4. Place the water pocket (3×3 block centered at (60, 40)).
    grid.fill_rect(59, 39, 3, 3, Material::Water);

    // 5. Mark all non-Earth, non-Vacuum cells as active so the scheduler
    //    picks them up on the initial seed pass.
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.get_mut(x, y);
            match cell.material {
                Material::Earth | Material::Vacuum => {
                    cell.active = false;
                }
                _ => {
                    cell.active = true;
                }
            }
        }
    }
}

/// Seeds the scheduler from the grid's active cells.
///
/// Thin wrapper so callers don't need to know about
/// `Scheduler::seed_from_grid` directly.
pub fn seed_scenario(scheduler: &mut Scheduler, grid: &Grid) {
    scheduler.seed_from_grid(grid);
}
