mod cell;
mod grid;
mod renderer;
mod rules;
mod scenario;
mod scheduler;

use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

use grid::Grid;
use renderer::Renderer;
use scheduler::Scheduler;

const GRID_W: usize = 120;
const GRID_H: usize = 120;
const CELL_SIZE: usize = 6;
const MAX_CELLS_PER_TICK: usize = 6000;

fn main() {
    // simulation state
    let mut grid = Grid::new(GRID_W, GRID_H);
    scenario::dome_test(&mut grid);

    let mut scheduler = Scheduler::new(GRID_W * GRID_H);
    scenario::seed_scenario(&mut scheduler, &grid);

    // window + renderer
    let event_loop = EventLoop::new().unwrap();

    let window_size = LogicalSize::new(
        (GRID_W * CELL_SIZE) as u32,
        (GRID_H * CELL_SIZE) as u32,
    );
    let window = WindowBuilder::new()
        .with_title("crucible dome test")
        .with_inner_size(window_size)
        .with_min_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window, GRID_W, GRID_H, CELL_SIZE);
    let mut tick_count: u64 = 0;

    // event loop
    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    // close
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }

                    // escape key
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => {
                        elwt.exit();
                    }

                    // redraw requested
                    WindowEvent::RedrawRequested => {
                        // advance simulation
                        grid.tick(&mut scheduler, MAX_CELLS_PER_TICK);
                        tick_count += 1;

                        // Debug probe: print temperature of cell (60, 50)
                        // every 100 ticks.
                        if tick_count % 100 == 0 {
                            let probe = grid.get(60, 50);
                            println!(
                                "[tick {}] cell(60,50): temp={:.2} pressure={:.2} mat={:?}",
                                tick_count,
                                probe.temperature,
                                probe.pressure,
                                probe.material,
                            );
                        }

                        // render
                        renderer.draw(&grid);
                        renderer.render();

                        // request another frame immediately
                        window.request_redraw();
                    }

                    _ => {}
                },

                // kick off the first redraw after the window appears
                Event::AboutToWait => {
                    window.request_redraw();
                }

                _ => {}
            }
        })
        .unwrap();
}
