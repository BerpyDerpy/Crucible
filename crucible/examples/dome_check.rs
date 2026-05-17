use crucible::{Cell, Grid, Material};

fn main() {
    let w = 80;
    let h = 80;
    let mut grid = Grid::new(w, h);
    let cx = 40.0_f32;
    let cy = 40.0_f32;
    let r = 30.0_f32;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if (d - r).abs() < 1.0 {
                grid.set(x, y, Cell::new(Material::Earth));
            }
        }
    }
    // big fire wall on the west side of the interior
    for y in 25..55 {
        grid.set(15, y, Cell::new(Material::Fire));
    }
    // water blob on east side
    for y in 38..43 {
        for x in 60..65 {
            grid.set(x, y, Cell::new(Material::Water));
        }
    }

    let water_probe = (62, 40);
    let air_probe = (40, 40);
    let earth_probe = (10, 40);
    let fire_probe = (15, 40);

    println!(
        "tick | fire.T  | earth.T | air.T   | water.T  PE      mat"
    );
    for t in 1..=1200 {
        grid.tick();
        if t % 50 == 0 {
            let f = grid.get(fire_probe.0, fire_probe.1);
            let e = grid.get(earth_probe.0, earth_probe.1);
            let a = grid.get(air_probe.0, air_probe.1);
            let w = grid.get(water_probe.0, water_probe.1);
            println!(
                "{:>4} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1}  {:>6.2}  {:?}",
                t, f.temperature, e.temperature, a.temperature, w.temperature, w.phase_energy, w.material
            );
        }
    }

    println!("\n--- water boil close-up (tick-by-tick near transition) ---");
    println!("tick | T       PE     mat");
    let mut grid2 = Grid::new(20, 20);
    // sealed 18x18 air pocket with earth walls, fire bar bottom, water blob top
    for x in 0..20 { grid2.set(x, 0, Cell::new(Material::Earth)); grid2.set(x, 19, Cell::new(Material::Earth)); }
    for y in 0..20 { grid2.set(0, y, Cell::new(Material::Earth)); grid2.set(19, y, Cell::new(Material::Earth)); }
    for x in 5..15 { grid2.set(x, 17, Cell::new(Material::Fire)); }
    for x in 9..11 { for y in 4..6 { grid2.set(x, y, Cell::new(Material::Water)); } }

    // ticks 1..1500
    let probe = (9, 4);
    let mut last_mat = Material::Water;
    let mut flips = 0;
    for t in 1..=2000 {
        grid2.tick();
        let c = grid2.get(probe.0, probe.1);
        if c.material != last_mat {
            flips += 1;
            println!("FLIP at tick {}: {:?} -> {:?} (T={:.2} PE={:.2})", t, last_mat, c.material, c.temperature, c.phase_energy);
            last_mat = c.material;
        }
        if t % 100 == 0 {
            println!("{:>4} | {:>6.2}  {:>6.2}  {:?}", t, c.temperature, c.phase_energy, c.material);
        }
    }
    println!("\ntotal phase flips at probe over 2000 ticks: {}", flips);
}
