use piston::{self, RenderEvent, UpdateEvent};

pub struct App {
    gl: opengl_graphics::GlGraphics,
    simulation: logicgates::simulation::Simulation,
    inputs: std::iter::Cycle<std::vec::IntoIter<Vec<bool>>>,
    current_input: Vec<bool>,
    wait: f64,
}

impl App {
    fn new(gl: opengl_graphics::GlGraphics, simulation: logicgates::simulation::Simulation) -> App {
        let mut inputs = logicgates::utils::enumerate_inputs(simulation.circuit.num_inputs).into_iter().cycle();
        let first_input = inputs.next().unwrap();
        App { gl, simulation, inputs, current_input: first_input, wait: 0.0 }
    }

    fn render(&mut self, render_args: &piston::RenderArgs) {
        self.simulation.render(&mut self.gl, render_args, &self.current_input);
    }

    fn update(&mut self, update_args: &piston::UpdateArgs) {
        if self.wait > 0.2 {
            self.wait = 0.0;
            self.current_input = self.inputs.next().unwrap();
        }
        self.wait += update_args.dt;

        self.simulation.update_positions_evolution();
    }
}

fn main() {
    let opengl = opengl_graphics::OpenGL::V3_2;

    let mut window: glutin_window::GlutinWindow = piston::WindowSettings::new("logic gates", [1280, 720]).graphics_api(opengl).resizable(true).samples(4).exit_on_esc(true).build().unwrap();

    let mut app = App::new(
        opengl_graphics::GlGraphics::new(opengl),
        logicgates::simulation::Simulation::new(logicgates::circuit::Circuit {
            num_inputs: 10,
            gates: vec![
                logicgates::circuit::Gate::And(logicgates::circuit::Value::Arg(0), logicgates::circuit::Value::Arg(1)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(0, 0), logicgates::circuit::Value::Arg(2)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(1, 0), logicgates::circuit::Value::Arg(3)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(2, 0), logicgates::circuit::Value::Arg(4)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(3, 0), logicgates::circuit::Value::Arg(5)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(4, 0), logicgates::circuit::Value::Arg(6)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(5, 0), logicgates::circuit::Value::Arg(7)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(6, 0), logicgates::circuit::Value::Arg(8)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(7, 0), logicgates::circuit::Value::Arg(9)),
            ],
            outputs: vec![logicgates::circuit::Value::GateValue(8, 0)],
        }),
    );

    let mut events = piston::Events::new(piston::EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
