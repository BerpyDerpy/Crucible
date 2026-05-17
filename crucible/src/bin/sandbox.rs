use crucible::cell::{Cell, Material, Vec2};
use crucible::entity::{Entity, step as step_entity};
use crucible::grid::Grid;
use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2 as EVec2};

const GRID_W: usize = 120;
const GRID_H: usize = 90;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "crucible sandbox",
        native_options,
        Box::new(|_cc| Ok(Box::new(SandboxApp::new()))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BrushMaterial {
    Air,
    Water,
    Fire,
    Earth,
    Ice,
    Steam,
}

impl BrushMaterial {
    fn to_material(self) -> Material {
        match self {
            BrushMaterial::Air => Material::Air,
            BrushMaterial::Water => Material::Water,
            BrushMaterial::Fire => Material::Fire,
            BrushMaterial::Earth => Material::Earth,
            BrushMaterial::Ice => Material::Ice,
            BrushMaterial::Steam => Material::Steam,
        }
    }

    fn label(self) -> &'static str {
        match self {
            BrushMaterial::Air => "Air",
            BrushMaterial::Water => "Water",
            BrushMaterial::Fire => "Fire",
            BrushMaterial::Earth => "Earth",
            BrushMaterial::Ice => "Ice",
            BrushMaterial::Steam => "Steam",
        }
    }

    fn default_temp(self) -> f32 {
        match self {
            BrushMaterial::Fire => 1400.0,
            BrushMaterial::Ice => 260.0,
            BrushMaterial::Steam => 400.0,
            _ => self.to_material().props().equilibrium_temp,
        }
    }

    fn all() -> &'static [BrushMaterial] {
        &[
            BrushMaterial::Air,
            BrushMaterial::Water,
            BrushMaterial::Fire,
            BrushMaterial::Earth,
            BrushMaterial::Ice,
            BrushMaterial::Steam,
        ]
    }
}

struct SandboxApp {
    grid: Grid,
    entities: Vec<Entity>,
    brush: BrushMaterial,
    brush_radius: i32,
    playing: bool,
    ticks_per_frame: u32,
    step_once: bool,
    show_temperature: bool,
    show_pressure: bool,
    show_velocity: bool,
    show_entities: bool,
    selected: Option<(usize, usize)>,
    spawn_mode: bool,
    tick_count: u64,
    painted_this_stroke: std::collections::HashSet<(usize, usize)>,
}

impl SandboxApp {
    fn new() -> Self {
        let mut grid = Grid::new(GRID_W, GRID_H);
        for x in 0..GRID_W {
            grid.set(x, GRID_H - 1, Cell::new(Material::Earth));
            grid.set(x, GRID_H - 2, Cell::new(Material::Earth));
        }
        Self {
            grid,
            entities: Vec::new(),
            brush: BrushMaterial::Water,
            brush_radius: 2,
            playing: false,
            ticks_per_frame: 1,
            step_once: false,
            show_temperature: false,
            show_pressure: false,
            show_velocity: false,
            show_entities: true,
            selected: None,
            spawn_mode: false,
            tick_count: 0,
            painted_this_stroke: std::collections::HashSet::new(),
        }
    }

    fn fill_with_air(&mut self) {
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                self.grid.set(x, y, Cell::new(Material::Air));
            }
        }
        self.entities.clear();
        self.tick_count = 0;
    }

    fn paint_at(&mut self, cx: i32, cy: i32) {
        let r = self.brush_radius;
        let mat = self.brush.to_material();
        let temp = self.brush.default_temp();
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let x = cx + dx;
                let y = cy + dy;
                if x < 0 || y < 0 || x >= self.grid.width() as i32 || y >= self.grid.height() as i32 {
                    continue;
                }
                let xy = (x as usize, y as usize);
                if !self.painted_this_stroke.insert(xy) {
                    continue;
                }
                let cell = Cell::new(mat).with_temperature(temp);
                self.grid.set(xy.0, xy.1, cell);
            }
        }
    }

    fn spawn_entity_at(&mut self, x: f32, y: f32) {
        let size = Vec2::new(3.0, 4.0);
        let pos = Vec2::new(x - size.x * 0.5, y - size.y * 0.5);
        self.entities.push(Entity::new(pos, size, 4.0));
    }

    fn step_sim(&mut self, ticks: u32) {
        let cfg = self.grid.config;
        for _ in 0..ticks {
            self.grid.tick();
            for ent in self.entities.iter_mut() {
                step_entity(ent, &self.grid, &cfg);
            }
            self.tick_count = self.tick_count.wrapping_add(1);
        }
    }
}

fn material_base_color(m: Material) -> Color32 {
    match m {
        Material::Air => Color32::from_rgb(20, 22, 30),
        Material::Water => Color32::from_rgb(40, 90, 200),
        Material::Steam => Color32::from_rgb(190, 200, 215),
        Material::Fire => Color32::from_rgb(230, 90, 30),
        Material::Earth => Color32::from_rgb(95, 70, 50),
        Material::Ice => Color32::from_rgb(170, 215, 240),
    }
}

fn cell_render_color(cell: &Cell) -> Color32 {
    let base = material_base_color(cell.material);
    let eq = cell.props().equilibrium_temp;
    let delta = (cell.temperature - eq) / 600.0;
    let delta = delta.clamp(-1.0, 1.0);
    let [r, g, b, _] = base.to_array();
    if delta > 0.0 {
        let t = delta;
        let nr = (r as f32 * (1.0 - t * 0.4) + 255.0 * t * 0.9) as u8;
        let ng = (g as f32 * (1.0 - t * 0.6) + 140.0 * t) as u8;
        let nb = (b as f32 * (1.0 - t * 0.8)) as u8;
        Color32::from_rgb(nr, ng, nb)
    } else if delta < 0.0 {
        let t = -delta;
        let nr = (r as f32 * (1.0 - t * 0.6)) as u8;
        let ng = (g as f32 * (1.0 - t * 0.4) + 80.0 * t) as u8;
        let nb = (b as f32 * (1.0 - t * 0.3) + 230.0 * t) as u8;
        Color32::from_rgb(nr, ng, nb)
    } else {
        base
    }
}

fn heatmap_temp(t: f32) -> Color32 {
    let n = ((t - 200.0) / 1400.0).clamp(0.0, 1.0);
    let r = (n * 255.0) as u8;
    let g = ((1.0 - (n - 0.5).abs() * 2.0).max(0.0) * 200.0) as u8;
    let b = ((1.0 - n) * 200.0) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, 140)
}

fn heatmap_pressure(p: f32) -> Color32 {
    let delta = (p - 101_325.0) / 80_000.0;
    let n = delta.clamp(-1.0, 1.0);
    if n >= 0.0 {
        let a = (n * 200.0) as u8;
        Color32::from_rgba_unmultiplied(255, 60, 60, a)
    } else {
        let a = (-n * 200.0) as u8;
        Color32::from_rgba_unmultiplied(60, 120, 255, a)
    }
}

impl eframe::App for SandboxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.playing {
            self.step_sim(self.ticks_per_frame);
            ctx.request_repaint();
        } else if self.step_once {
            self.step_sim(1);
            self.step_once = false;
        }

        egui::SidePanel::left("palette").min_width(170.0).show(ctx, |ui| {
            ui.heading("Brush");
            for m in BrushMaterial::all() {
                ui.selectable_value(&mut self.brush, *m, m.label());
            }
            ui.separator();
            ui.label("Brush radius");
            ui.add(egui::Slider::new(&mut self.brush_radius, 0..=10));

            ui.separator();
            ui.heading("Simulation");
            ui.horizontal(|ui| {
                if ui.button(if self.playing { "Pause" } else { "Play" }).clicked() {
                    self.playing = !self.playing;
                }
                if ui.button("Step").clicked() {
                    self.step_once = true;
                }
            });
            ui.label("Ticks per frame");
            ui.add(egui::Slider::new(&mut self.ticks_per_frame, 1..=20));
            ui.label(format!("tick: {}", self.tick_count));
            if ui.button("Fill with air").clicked() {
                self.fill_with_air();
            }

            ui.separator();
            ui.heading("Overlays");
            ui.checkbox(&mut self.show_temperature, "Temperature");
            ui.checkbox(&mut self.show_pressure, "Pressure");
            ui.checkbox(&mut self.show_velocity, "Velocity field");
            ui.checkbox(&mut self.show_entities, "Entity boxes");

            ui.separator();
            ui.heading("Entities");
            let spawn_label = if self.spawn_mode { "Cancel spawn" } else { "Spawn on click" };
            if ui.button(spawn_label).clicked() {
                self.spawn_mode = !self.spawn_mode;
            }
            ui.label(format!("count: {}", self.entities.len()));
            if ui.button("Clear entities").clicked() {
                self.entities.clear();
            }

            ui.separator();
            ui.heading("Physics");
            let cfg = self.grid.config_mut();
            ui.label("Gravity");
            ui.add(egui::Slider::new(&mut cfg.gravity.y, -20.0..=30.0));
            ui.label("Pressure → velocity");
            ui.add(
                egui::Slider::new(&mut cfg.pressure_to_velocity, 0.0..=1.0e-4)
                    .logarithmic(true),
            );
            ui.label("Advection damping");
            ui.add(egui::Slider::new(&mut cfg.advection_damping, 0.5..=1.0));
            ui.label("Steam pressure spike");
            ui.add(egui::Slider::new(&mut cfg.steam_pressure_spike, 0.0..=200_000.0));
            ui.label("Entity drag");
            ui.add(egui::Slider::new(&mut cfg.entity_drag, 0.0..=1.0));
            ui.label("Entity pressure force");
            ui.add(
                egui::Slider::new(&mut cfg.entity_pressure_force, 0.0..=1.0e-2)
                    .logarithmic(true),
            );
            ui.label("Temp diffusion rate");
            ui.add(egui::Slider::new(&mut cfg.temp_diffusion_rate, 0.0..=1.0));
        });

        egui::SidePanel::right("inspector").min_width(220.0).show(ctx, |ui| {
            ui.heading("Cell inspector");
            if let Some((x, y)) = self.selected {
                let c = self.grid.get(x, y);
                ui.label(format!("({}, {})", x, y));
                ui.label(format!("material: {:?}", c.material));
                ui.label(format!("T: {:.2} K", c.temperature));
                ui.label(format!("P: {:.1} Pa", c.pressure));
                ui.label(format!("v: ({:.3}, {:.3})", c.velocity.x, c.velocity.y));
                ui.label(format!("density: {:.2}", c.density));
            } else {
                ui.label("Right-click a cell to inspect.");
            }
            ui.separator();
            ui.heading("Stats");
            ui.label(format!("entities: {}", self.entities.len()));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let gw = self.grid.width() as f32;
            let gh = self.grid.height() as f32;
            let cell_size = (avail.x / gw).min(avail.y / gh).max(1.0);
            let canvas_size = EVec2::new(gw * cell_size, gh * cell_size);
            let (rect, response) =
                ui.allocate_exact_size(canvas_size, Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            for y in 0..self.grid.height() {
                for x in 0..self.grid.width() {
                    let c = self.grid.get(x, y);
                    let p0 = Pos2::new(
                        rect.min.x + x as f32 * cell_size,
                        rect.min.y + y as f32 * cell_size,
                    );
                    let cell_rect =
                        Rect::from_min_size(p0, EVec2::splat(cell_size));
                    painter.rect_filled(cell_rect, 0.0, cell_render_color(c));

                    if self.show_temperature {
                        painter.rect_filled(cell_rect, 0.0, heatmap_temp(c.temperature));
                    }
                    if self.show_pressure {
                        painter.rect_filled(cell_rect, 0.0, heatmap_pressure(c.pressure));
                    }
                }
            }

            if self.show_velocity {
                let stride = 3usize;
                for y in (0..self.grid.height()).step_by(stride) {
                    for x in (0..self.grid.width()).step_by(stride) {
                        let c = self.grid.get(x, y);
                        let v = c.velocity;
                        let mag = v.length();
                        if mag < 0.05 {
                            continue;
                        }
                        let cx = rect.min.x + (x as f32 + 0.5) * cell_size;
                        let cy = rect.min.y + (y as f32 + 0.5) * cell_size;
                        let scale = (cell_size * stride as f32 * 0.4) / mag.max(1.0);
                        let ex = cx + v.x * scale;
                        let ey = cy + v.y * scale;
                        let start = Pos2::new(cx, cy);
                        let end = Pos2::new(ex, ey);
                        let stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 180));
                        painter.line_segment([start, end], stroke);
                        painter.circle_filled(end, 1.5, Color32::from_rgb(255, 255, 180));
                    }
                }
            }

            if self.show_entities {
                for ent in &self.entities {
                    let min = Pos2::new(
                        rect.min.x + ent.position.x * cell_size,
                        rect.min.y + ent.position.y * cell_size,
                    );
                    let max = Pos2::new(
                        min.x + ent.size.x * cell_size,
                        min.y + ent.size.y * cell_size,
                    );
                    let ebox = Rect::from_min_max(min, max);
                    painter.rect_filled(ebox, 0.0, Color32::from_rgba_unmultiplied(255, 220, 80, 180));
                    painter.rect_stroke(
                        ebox,
                        0.0,
                        Stroke::new(1.5, Color32::from_rgb(40, 30, 0)),
                    );
                }
            }

            let pointer = response.interact_pointer_pos();
            if let Some(pos) = pointer {
                let gx_f = (pos.x - rect.min.x) / cell_size;
                let gy_f = (pos.y - rect.min.y) / cell_size;
                let gx = gx_f.floor() as i32;
                let gy = gy_f.floor() as i32;
                let in_bounds =
                    gx >= 0 && gy >= 0 && gx < self.grid.width() as i32 && gy < self.grid.height() as i32;
                if in_bounds {
                    if response.clicked_by(egui::PointerButton::Secondary) {
                        self.selected = Some((gx as usize, gy as usize));
                    }
                    if response.clicked() || response.dragged() {
                        if self.spawn_mode && response.clicked() {
                            self.spawn_entity_at(gx_f, gy_f);
                            self.spawn_mode = false;
                        } else if !self.spawn_mode {
                            self.paint_at(gx, gy);
                        }
                    }
                }
            }
            if response.drag_stopped() || ctx.input(|i| i.pointer.any_released()) {
                self.painted_this_stroke.clear();
            }

            if let Some(pos) = response.hover_pos() {
                let gx_f = (pos.x - rect.min.x) / cell_size;
                let gy_f = (pos.y - rect.min.y) / cell_size;
                let cx = rect.min.x + gx_f.floor() * cell_size + cell_size * 0.5;
                let cy = rect.min.y + gy_f.floor() * cell_size + cell_size * 0.5;
                let radius_px = (self.brush_radius as f32 + 0.5) * cell_size;
                painter.circle_stroke(
                    Pos2::new(cx, cy),
                    radius_px,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
                );
            }
        });
    }
}
