//! Defines the cell state types.

/// The material occupying a cell in the simulation grid.
#[derive(Debug, Clone)]
pub enum Material {
    Vacuum,
    Air,
    Earth,
    Fire,
    Water,
    Steam,
}

impl Material {
    /// Returns the thermal conductivity constant for this material.
    pub fn conductivity(&self) -> f32 {
        match self {
            Material::Vacuum => 0.0,
            Material::Air => 0.1,
            Material::Earth => 0.02,
            Material::Fire => 0.4,
            Material::Water => 0.3,
            Material::Steam => 0.15,
        }
    }

    /// Returns true if this material behaves as a gas.
    pub fn is_gas(&self) -> bool {
        matches!(self, Material::Air | Material::Fire | Material::Steam | Material::Vacuum)
    }

    /// Returns true if this material behaves as a liquid.
    pub fn is_liquid(&self) -> bool {
        matches!(self, Material::Water)
    }
}

/// A single cell in the simulation grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub material: Material,
    /// Abstract temperature units; ambient = 20.0
    pub temperature: f32,
    /// Relative pressure; ambient = 1.0
    pub pressure: f32,
    /// Derived from material, stored for fast lookup.
    pub density: f32,
    /// Mana density; 0.0 = none.
    pub mana_density: f32,
    /// Whether this cell is in the scheduler.
    pub active: bool,
}

impl Cell {
    /// Returns a cell with sensible physical defaults for the given material.
    pub fn default_for(material: Material) -> Cell {
        let (temperature, pressure, density) = match &material {
            Material::Vacuum => (0.0, 0.0, 0.0),
            Material::Air => (20.0, 1.0, 0.3),
            Material::Earth => (20.0, 1.0, 2.5),
            Material::Fire => (800.0, 1.2, 0.1),
            Material::Water => (20.0, 1.0, 1.0),
            Material::Steam => (110.0, 2.5, 0.05),
        };

        Cell {
            material,
            temperature,
            pressure,
            density,
            mana_density: 0.0,
            active: false,
        }
    }
}
