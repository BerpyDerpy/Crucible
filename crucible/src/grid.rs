use crate::cell::Cell;
use crate::rules::{NeighborView, Neighborhood, PhysicsConfig, RuleOutput, apply_all, resolve_phase};

pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    writes: Vec<RuleOutput>,
    pub config: PhysicsConfig,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let count = width * height;
        Self {
            width,
            height,
            cells: vec![Cell::default(); count],
            writes: vec![RuleOutput::default(); count],
            config: PhysicsConfig::default(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.index(x, y)]
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn config_mut(&mut self) -> &mut PhysicsConfig {
        &mut self.config
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let idx = self.index(x, y);
        self.cells[idx] = cell;
    }

    fn neighborhood_at(&self, x: usize, y: usize) -> Neighborhood {
        let view = |dx: i32, dy: i32| -> NeighborView {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if !self.in_bounds(nx, ny) {
                return NeighborView::ABSENT;
            }
            NeighborView {
                cell: self.cells[self.index(nx as usize, ny as usize)],
                present: true,
            }
        };
        Neighborhood {
            north: view(0, -1),
            south: view(0, 1),
            east: view(1, 0),
            west: view(-1, 0),
        }
    }

    pub fn tick(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let cell = self.cells[idx];
                let neighborhood = self.neighborhood_at(x, y);
                self.writes[idx] = apply_all(&cell, &neighborhood, &self.config);
            }
        }

        for idx in 0..self.cells.len() {
            let out = self.writes[idx];
            let cell = &mut self.cells[idx];
            cell.temperature += out.delta_temperature;
            cell.pressure += out.delta_pressure;
            cell.velocity = cell.velocity.add(out.delta_velocity);

            if let Some(new_mat) = resolve_phase(cell, &self.config) {
                cell.material = new_mat;
                cell.density = new_mat.props().density;
            }

            cell.temperature = cell.temperature.clamp(0.0, 5000.0);
            cell.pressure = cell.pressure.clamp(0.0, self.config.max_pressure);
            let speed = cell.velocity.length();
            const MAX_SPEED: f32 = 50.0;
            if speed > MAX_SPEED {
                cell.velocity = cell.velocity.scale(MAX_SPEED / speed);
            }
        }
    }

    pub fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick();
        }
    }
}
