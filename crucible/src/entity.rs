use crate::cell::Vec2;
use crate::grid::Grid;
use crate::rules::PhysicsConfig;

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
}

impl Entity {
    pub fn new(position: Vec2, size: Vec2, mass: f32) -> Self {
        Self {
            position,
            size,
            velocity: Vec2::ZERO,
            mass,
        }
    }

    pub fn min(&self) -> Vec2 {
        self.position
    }

    pub fn max(&self) -> Vec2 {
        self.position.add(self.size)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ForceAccum {
    pressure: Vec2,
    drag: Vec2,
    gravity: Vec2,
}

impl ForceAccum {
    fn total(&self) -> Vec2 {
        self.pressure.add(self.drag).add(self.gravity)
    }
}

fn overlap_1d(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    (a_max.min(b_max) - a_min.max(b_min)).max(0.0)
}

fn sample_forces(entity: &Entity, grid: &Grid, cfg: &PhysicsConfig) -> ForceAccum {
    let mut acc = ForceAccum::default();
    let emin = entity.min();
    let emax = entity.max();

    let x0 = emin.x.floor() as i32;
    let y0 = emin.y.floor() as i32;
    let x1 = (emax.x.ceil() as i32) - 1;
    let y1 = (emax.y.ceil() as i32) - 1;

    for cy in y0..=y1 {
        for cx in x0..=x1 {
            if !grid.in_bounds(cx, cy) {
                continue;
            }
            let ox = overlap_1d(emin.x, emax.x, cx as f32, (cx + 1) as f32);
            let oy = overlap_1d(emin.y, emax.y, cy as f32, (cy + 1) as f32);
            let area = ox * oy;
            if area <= 0.0 {
                continue;
            }
            let cell = grid.get(cx as usize, cy as usize);
            let props = cell.props();
            if props.is_solid {
                continue;
            }

            let rel = cell.velocity.sub(entity.velocity);
            let drag_k = cfg.entity_drag * props.density * area;
            acc.drag = acc.drag.add(rel.scale(drag_k));

            for (dx, dy, dir) in [
                (-1_i32, 0_i32, Vec2::new(-1.0, 0.0)),
                (1, 0, Vec2::new(1.0, 0.0)),
                (0, -1, Vec2::new(0.0, -1.0)),
                (0, 1, Vec2::new(0.0, 1.0)),
            ] {
                let nx = cx + dx;
                let ny = cy + dy;
                let neighbor_p = if grid.in_bounds(nx, ny) {
                    grid.get(nx as usize, ny as usize).pressure
                } else {
                    cell.pressure
                };
                let gradient = cell.pressure - neighbor_p;
                acc.pressure = acc.pressure.add(dir.scale(gradient * area * cfg.entity_pressure_force));
            }
        }
    }

    acc.gravity = cfg.gravity.scale(entity.mass);
    acc
}

fn resolve_axis_collision(
    entity: &mut Entity,
    grid: &Grid,
    axis_x: bool,
    delta: f32,
) {
    if axis_x {
        entity.position.x += delta;
    } else {
        entity.position.y += delta;
    }

    let emin = entity.min();
    let emax = entity.max();
    let x0 = emin.x.floor() as i32;
    let y0 = emin.y.floor() as i32;
    let x1 = (emax.x.ceil() as i32) - 1;
    let y1 = (emax.y.ceil() as i32) - 1;

    let mut max_pen = 0.0_f32;
    let mut pen_sign = 0.0_f32;

    for cy in y0..=y1 {
        for cx in x0..=x1 {
            let solid = if !grid.in_bounds(cx, cy) {
                true
            } else {
                grid.get(cx as usize, cy as usize).props().is_solid
            };
            if !solid {
                continue;
            }
            let cell_min_x = cx as f32;
            let cell_max_x = (cx + 1) as f32;
            let cell_min_y = cy as f32;
            let cell_max_y = (cy + 1) as f32;

            let ox = overlap_1d(emin.x, emax.x, cell_min_x, cell_max_x);
            let oy = overlap_1d(emin.y, emax.y, cell_min_y, cell_max_y);
            if ox <= 0.0 || oy <= 0.0 {
                continue;
            }

            if axis_x {
                let pen = ox;
                if pen > max_pen {
                    max_pen = pen;
                    let entity_center_x = (emin.x + emax.x) * 0.5;
                    let cell_center_x = (cell_min_x + cell_max_x) * 0.5;
                    pen_sign = if entity_center_x < cell_center_x { -1.0 } else { 1.0 };
                }
            } else {
                let pen = oy;
                if pen > max_pen {
                    max_pen = pen;
                    let entity_center_y = (emin.y + emax.y) * 0.5;
                    let cell_center_y = (cell_min_y + cell_max_y) * 0.5;
                    pen_sign = if entity_center_y < cell_center_y { -1.0 } else { 1.0 };
                }
            }
        }
    }

    if max_pen > 0.0 {
        if axis_x {
            entity.position.x += pen_sign * max_pen;
            entity.velocity.x = 0.0;
        } else {
            entity.position.y += pen_sign * max_pen;
            entity.velocity.y = 0.0;
        }
    }
}

fn substep_move(entity: &mut Entity, grid: &Grid, axis_x: bool, total_delta: f32) {
    let steps = total_delta.abs().ceil().max(1.0) as usize;
    let per = total_delta / steps as f32;
    for _ in 0..steps {
        resolve_axis_collision(entity, grid, axis_x, per);
        let stopped = if axis_x { entity.velocity.x == 0.0 } else { entity.velocity.y == 0.0 };
        if stopped {
            break;
        }
    }
}

pub fn step(entity: &mut Entity, grid: &Grid, cfg: &PhysicsConfig) {
    let forces = sample_forces(entity, grid, cfg);
    let inv_mass = 1.0 / entity.mass.max(1e-6);
    let accel = forces.total().scale(inv_mass);

    entity.velocity = entity.velocity.add(accel.scale(cfg.dt));

    let dx = entity.velocity.x * cfg.dt;
    substep_move(entity, grid, true, dx);
    let dy = entity.velocity.y * cfg.dt;
    substep_move(entity, grid, false, dy);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{Cell, Material};

    fn make_grid(w: usize, h: usize) -> Grid {
        Grid::new(w, h)
    }

    #[test]
    fn gravity_pulls_entity_down_in_air() {
        let grid = make_grid(10, 10);
        let cfg = PhysicsConfig::default();
        let mut e = Entity::new(Vec2::new(4.0, 1.0), Vec2::new(2.0, 2.0), 5.0);
        let y0 = e.position.y;
        for _ in 0..20 {
            step(&mut e, &grid, &cfg);
        }
        assert!(e.position.y > y0, "entity should fall: y0={} y1={}", y0, e.position.y);
    }

    #[test]
    fn earth_floor_stops_entity() {
        let mut grid = make_grid(10, 10);
        for x in 0..10 {
            grid.set(x, 9, Cell::new(Material::Earth));
        }
        let cfg = PhysicsConfig::default();
        let mut e = Entity::new(Vec2::new(4.0, 1.0), Vec2::new(2.0, 2.0), 5.0);
        for _ in 0..200 {
            step(&mut e, &grid, &cfg);
        }
        assert!(e.position.y + e.size.y <= 9.0 + 1e-3, "entity should rest on or above y=9, got max_y={}", e.position.y + e.size.y);
        assert!(e.velocity.y.abs() < 1e-3, "vy should be zero after landing, got {}", e.velocity.y);
    }

    #[test]
    fn pressure_gradient_pushes_entity_up() {
        let mut grid = make_grid(10, 10);
        for x in 0..10 {
            let mut c = Cell::new(Material::Air);
            c.pressure = 200_000.0;
            grid.set(x, 6, c);
        }
        let cfg = PhysicsConfig {
            gravity: Vec2::ZERO,
            ..PhysicsConfig::default()
        };
        let e = Entity::new(Vec2::new(4.0, 4.0), Vec2::new(2.0, 2.0), 1.0);
        let f = sample_forces(&e, &grid, &cfg);
        assert!(f.pressure.y < 0.0, "high pressure below should produce upward force, got {:?}", f.pressure);
    }
}