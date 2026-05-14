//! Implements cellular automata thermodynamic rules.
//!
//! Pure functions that compute the next state of individual cells.
//! Called by the main simulation loop each tick.

use crate::cell::{Cell, Material};

// ── Temperature diffusion ──────────────────────────────────────────────

/// Returns the new temperature for `cell` after one tick of heat diffusion.
///
/// Uses Newton's law of cooling: heat flux ∝ ΔT × average conductivity.
/// The average (rather than product) of conductivities models thermal
/// resistance in series — heat flow is limited by the worse conductor
/// but not annihilated by it.
///
/// Fire cells are treated as heat *sources*: they radiate outward but
/// don't absorb cold from neighbors.  Fire cooling is handled solely
/// by `decay_fire` (fuel depletion).
pub fn diffuse_temperature(cell: &Cell, neighbors: &[(i32, i32, &Cell)]) -> f32 {
    let mut delta = 0.0_f32;

    for &(_nx, _ny, neighbor) in neighbors {
        let diff = neighbor.temperature - cell.temperature;

        // Fire radiates heat outward but doesn't cool via diffusion.
        // Its temperature is governed by decay_fire (fuel running out).
        if matches!(cell.material, Material::Fire) && diff < 0.0 {
            continue;
        }

        let avg_cond = (cell.material.conductivity()
            + neighbor.material.conductivity()) / 2.0;
        delta += diff * avg_cond;
    }

    let mut new_temp = cell.temperature + delta * 0.35;

    // Fire is a sustained heat source: clamp its temperature floor so it
    // keeps pumping heat into neighbors even as heat diffuses outward.
    if matches!(cell.material, Material::Fire) {
        new_temp = new_temp.max(800.0);
    }

    new_temp.clamp(0.0, 200_000.0)
}


// ── Pressure diffusion ─────────────────────────────────────────────────

/// Returns the new pressure for `cell` after one tick of pressure equalization.
///
/// Only gas and liquid cells participate — solids (Earth, Ice) keep their
/// pressure unchanged.
pub fn diffuse_pressure(cell: &Cell, neighbors: &[(i32, i32, &Cell)]) -> f32 {
    // Earth and Ice are rigid; pressure doesn't diffuse through them.
    if matches!(cell.material, Material::Earth | Material::Ice) {
        return cell.pressure;
    }

    let mut delta = 0.0_f32;

    for &(_nx, _ny, neighbor) in neighbors {
        delta += (neighbor.pressure - cell.pressure) * 0.3;
    }

    (cell.pressure + delta).clamp(0.0, 20.0)
}

// ── Phase transitions ──────────────────────────────────────────────────

/// Checks whether `cell` should undergo a phase change this tick.
///
/// Returns `Some(new_material)` if a transition is triggered, otherwise `None`.
pub fn check_phase_transition(cell: &Cell) -> Option<Material> {
    match cell.material {
        Material::Water if cell.temperature > 100.0 => Some(Material::Steam),
        Material::Water if cell.temperature < 0.0 => Some(Material::Ice),
        Material::Steam if cell.temperature < 80.0 => Some(Material::Water),
        Material::Fire if cell.temperature < 50.0 => Some(Material::Air),
        Material::Earth if cell.temperature > 1200.0 => Some(Material::Lava),
        Material::Lava if cell.temperature < 200.0 => Some(Material::Earth),
        _ => None,
    }
}

/// Mutates `cell` in place to complete a phase transition.
///
/// Updates material and density, and applies pressure spikes/drops for
/// steam transitions (water→steam expands, steam→water contracts).
pub fn apply_phase_transition(cell: &mut Cell, new_material: Material) {
    let old_material = cell.material;
    cell.material = new_material;
    cell.density = Cell::default_for(new_material).density;

    // Steam expansion: sudden pressure spike when water boils.
    if new_material == Material::Steam {
        cell.pressure *= 8.0;
    }

    // Steam condensation: pressure collapses back down.
    if old_material == Material::Steam && new_material == Material::Water {
        cell.pressure /= 8.0;
    }
}

// ── Fire decay ─────────────────────────────────────────────────────────

/// Fire cells lose temperature each tick — they burn out without fuel.
///
/// Only meaningful for `Material::Fire` cells; calling on others is a no-op
/// in practice, but the simulation loop should gate this.
pub fn decay_fire(cell: &mut Cell) {
    // Very slow decay so the fire sustains long enough for heat to
    // propagate across the dome and boil the water pocket.
    // At 0.9998 per tick a 100 000° fire halves roughly every ~3 500 ticks.
    cell.temperature *= 0.9998;
}
