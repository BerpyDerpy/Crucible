use crate::cell::{Cell, Material, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct PhysicsConfig {
    pub dt: f32,
    pub gravity: Vec2,
    pub temp_diffusion_rate: f32,
    pub pressure_diffusion_rate: f32,
    pub pressure_to_velocity: f32,
    pub advection_damping: f32,
    pub water_boil_temp: f32,
    pub water_latent_heat: f32,
    pub steam_pressure_spike: f32,
    pub steam_upward_impulse: f32,
    pub ice_melt_temp: f32,
    pub entity_drag: f32,
    pub entity_pressure_force: f32,
    pub max_pressure: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            dt: 1.0,
            gravity: Vec2::new(0.0, 9.81),
            temp_diffusion_rate: 0.18,
            pressure_diffusion_rate: 0.22,
            pressure_to_velocity: 1.5e-6,
            advection_damping: 0.92,
            water_boil_temp: 373.15,
            water_latent_heat: 60.0,
            steam_pressure_spike: 40_000.0,
            steam_upward_impulse: 6.0,
            ice_melt_temp: 273.15,
            entity_drag: 0.05,
            entity_pressure_force: 1.5e-4,
            max_pressure: 500_000.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuleOutput {
    pub delta_temperature: f32,
    pub delta_pressure: f32,
    pub delta_velocity: Vec2,
}

impl RuleOutput {
    pub fn merge(&mut self, other: RuleOutput) {
        self.delta_temperature += other.delta_temperature;
        self.delta_pressure += other.delta_pressure;
        self.delta_velocity = self.delta_velocity.add(other.delta_velocity);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NeighborView {
    pub cell: Cell,
    pub present: bool,
}

impl NeighborView {
    pub const ABSENT: NeighborView = NeighborView {
        cell: Cell {
            material: Material::Earth,
            temperature: 293.15,
            pressure: 101_325.0,
            velocity: Vec2::ZERO,
            density: 2500.0,
            phase_energy: 0.0,
        },
        present: false,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Neighborhood {
    pub north: NeighborView,
    pub south: NeighborView,
    pub east: NeighborView,
    pub west: NeighborView,
}

impl Neighborhood {
    pub fn iter(&self) -> [(NeighborView, Vec2); 4] {
        [
            (self.north, Vec2::new(0.0, -1.0)),
            (self.south, Vec2::new(0.0, 1.0)),
            (self.east, Vec2::new(1.0, 0.0)),
            (self.west, Vec2::new(-1.0, 0.0)),
        ]
    }
}

pub fn temperature_diffusion(cell: &Cell, n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    // Explicit-Euler diffusion on a 4-neighbor stencil. To stay stable the
    // per-neighbor coupling rate*k must be <= 0.25 (so the four contributions
    // sum to <= 1.0 and a cell can never overshoot the average of its
    // neighbors). We clamp here rather than trusting the user-facing
    // temp_diffusion_rate slider, so high-conductivity materials like earth
    // (k=1.5) don't oscillate.
    let self_k = cell.props().conductivity;
    let mut acc = 0.0_f32;
    for (nb, _) in n.iter() {
        if !nb.present {
            continue;
        }
        let k = (self_k + nb.cell.props().conductivity) * 0.5;
        let coupling = (cfg.temp_diffusion_rate * k * cfg.dt).min(0.25);
        acc += coupling * (nb.cell.temperature - cell.temperature);
    }
    RuleOutput {
        delta_temperature: acc,
        ..Default::default()
    }
}

pub fn pressure_diffusion(cell: &Cell, n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    if cell.props().is_solid {
        return RuleOutput::default();
    }
    let mut acc = 0.0_f32;
    for (nb, _) in n.iter() {
        if !nb.present {
            continue;
        }
        if nb.cell.props().is_solid {
            continue;
        }
        acc += nb.cell.pressure - cell.pressure;
    }
    RuleOutput {
        delta_pressure: acc * cfg.pressure_diffusion_rate * cfg.dt,
        ..Default::default()
    }
}

pub fn pressure_gradient_force(cell: &Cell, n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    if cell.props().is_solid {
        return RuleOutput::default();
    }
    let mut force = Vec2::ZERO;
    for (nb, dir) in n.iter() {
        if !nb.present {
            continue;
        }
        if nb.cell.props().is_solid {
            continue;
        }
        let gradient = cell.pressure - nb.cell.pressure;
        force = force.add(dir.scale(gradient));
    }
    let inv_mass = 1.0 / cell.density.max(0.01);
    RuleOutput {
        delta_velocity: force.scale(cfg.pressure_to_velocity * cfg.dt * inv_mass),
        ..Default::default()
    }
}

pub fn advection_damping(cell: &Cell, _n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    if cell.props().is_solid {
        return RuleOutput {
            delta_velocity: cell.velocity.scale(-1.0),
            ..Default::default()
        };
    }
    let damp = cfg.advection_damping.powf(cfg.dt);
    let target = cell.velocity.scale(damp);
    RuleOutput {
        delta_velocity: target.sub(cell.velocity),
        ..Default::default()
    }
}

pub fn gravity(cell: &Cell, n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    if cell.props().is_solid {
        return RuleOutput::default();
    }
    let mut ambient_density = 0.0_f32;
    let mut count = 0.0_f32;
    for (nb, _) in n.iter() {
        if !nb.present {
            continue;
        }
        if nb.cell.props().is_solid {
            continue;
        }
        ambient_density += nb.cell.density;
        count += 1.0;
    }
    let ambient = if count > 0.0 {
        ambient_density / count
    } else {
        cell.density
    };
    let buoyancy = (cell.density - ambient) / cell.density.max(0.01);
    RuleOutput {
        delta_velocity: cfg.gravity.scale(buoyancy * cfg.dt),
        ..Default::default()
    }
}

pub fn heat_generation(cell: &Cell, _n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    let rate = cell.props().heat_generation;
    if rate <= 0.0 {
        return RuleOutput::default();
    }
    let target = cell.props().equilibrium_temp;
    let headroom = (target - cell.temperature).max(0.0);
    RuleOutput {
        delta_temperature: rate.min(headroom) * cfg.dt,
        ..Default::default()
    }
}

pub fn apply_all(cell: &Cell, n: &Neighborhood, cfg: &PhysicsConfig) -> RuleOutput {
    let mut out = RuleOutput::default();
    out.merge(temperature_diffusion(cell, n, cfg));
    out.merge(pressure_diffusion(cell, n, cfg));
    out.merge(pressure_gradient_force(cell, n, cfg));
    out.merge(advection_damping(cell, n, cfg));
    out.merge(gravity(cell, n, cfg));
    out.merge(heat_generation(cell, n, cfg));
    out
}

/// Resolves phase boundaries with a latent-heat accumulator. Runs after all
/// deltas from `apply_all` have been written to the cell, so it sees the
/// post-diffusion temperature.
///
/// While a cell sits at the phase-change temperature, incoming heat fills
/// `phase_energy` instead of pushing temperature past the boundary. When the
/// buffer reaches the latent-heat threshold for a transition, the material
/// flips and any leftover energy carries through as a temperature delta in
/// the new phase. This stops the "flicker" of one-shot delta-on-transition.
pub fn resolve_phase(cell: &mut Cell, cfg: &PhysicsConfig) -> Option<Material> {
    match cell.material {
        Material::Water => {
            if cell.temperature > cfg.water_boil_temp {
                let excess = cell.temperature - cfg.water_boil_temp;
                cell.temperature = cfg.water_boil_temp;
                cell.phase_energy += excess;
                if cell.phase_energy >= cfg.water_latent_heat {
                    let carry = cell.phase_energy - cfg.water_latent_heat;
                    cell.phase_energy = 0.0;
                    cell.temperature = cfg.water_boil_temp + carry;
                    cell.pressure += cfg.steam_pressure_spike;
                    cell.velocity =
                        cell.velocity.add(Vec2::new(0.0, -cfg.steam_upward_impulse));
                    return Some(Material::Steam);
                }
            } else if cell.phase_energy > 0.0 {
                // cooled back below boil without finishing the transition;
                // give the energy back as temperature.
                cell.temperature += cell.phase_energy;
                cell.phase_energy = 0.0;
            }
        }
        Material::Steam => {
            if cell.temperature < cfg.water_boil_temp {
                let deficit = cfg.water_boil_temp - cell.temperature;
                cell.temperature = cfg.water_boil_temp;
                cell.phase_energy -= deficit;
                if cell.phase_energy <= -cfg.water_latent_heat {
                    let carry = -(cell.phase_energy + cfg.water_latent_heat);
                    cell.phase_energy = 0.0;
                    cell.temperature = cfg.water_boil_temp - carry;
                    return Some(Material::Water);
                }
            } else if cell.phase_energy < 0.0 {
                cell.temperature -= -cell.phase_energy;
                cell.phase_energy = 0.0;
            }
        }
        Material::Ice => {
            if cell.temperature > cfg.ice_melt_temp {
                let excess = cell.temperature - cfg.ice_melt_temp;
                cell.temperature = cfg.ice_melt_temp;
                cell.phase_energy += excess;
                let threshold = cfg.water_latent_heat * 0.4;
                if cell.phase_energy >= threshold {
                    let carry = cell.phase_energy - threshold;
                    cell.phase_energy = 0.0;
                    cell.temperature = cfg.ice_melt_temp + carry;
                    return Some(Material::Water);
                }
            } else if cell.phase_energy > 0.0 {
                cell.temperature += cell.phase_energy;
                cell.phase_energy = 0.0;
            }
        }
        _ => {}
    }
    None
}
