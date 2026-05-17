use crucible::cell::{Cell, Material};
use crucible::grid::Grid;

fn main() {
    let width = 12;
    let height = 12;
    let mut grid = Grid::new(width, height);

    for x in 0..width {
        grid.set(x, height - 1, Cell::new(Material::Earth));
    }

    let fire_row = height - 2;
    for x in 2..width - 2 {
        grid.set(
            x,
            fire_row,
            Cell::new(Material::Fire).with_temperature(1400.0),
        );
    }

    for y in (fire_row - 2)..fire_row {
        for x in 3..width - 3 {
            grid.set(x, y, Cell::new(Material::Water).with_temperature(290.0));
        }
    }

    let probes: [(&str, usize, usize); 4] = [
        ("fire   ", 6, fire_row),
        ("water_l", 6, fire_row - 1),
        ("water_u", 6, fire_row - 2),
        ("air_top", 6, 2),
    ];

    println!("=== crucible headless run: {}x{}, 500 ticks ===", width, height);
    print_probes(&grid, 0, &probes);

    for step in 1..=500 {
        grid.tick();
        if step % 50 == 0 {
            print_probes(&grid, step, &probes);
        }
    }
}

fn print_probes(grid: &Grid, step: usize, probes: &[(&str, usize, usize)]) {
    println!("\n-- tick {:>3} --", step);
    println!(
        "  {:<8} {:<8} {:>8} {:>10} {:>10} {:>10}",
        "label", "mat", "T(K)", "P(Pa)", "vx", "vy"
    );
    for (label, x, y) in probes {
        let c = grid.get(*x, *y);
        println!(
            "  {:<8} {:<8?} {:>8.2} {:>10.1} {:>10.3} {:>10.3}",
            label, c.material, c.temperature, c.pressure, c.velocity.x, c.velocity.y
        );
    }
}
