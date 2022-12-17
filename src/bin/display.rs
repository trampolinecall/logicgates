use piston::{self, RenderEvent, UpdateEvent};

pub struct App {
    gl: opengl_graphics::GlGraphics,
    circuit: logicgates::circuit::Circuit,
    locations: Vec<[f64; 2]>,
    inputs: std::iter::Cycle<std::vec::IntoIter<Vec<bool>>>,
    current_input: Vec<bool>,
    wait: f64,
}

impl App {
    fn render(&mut self, render_args: &piston::RenderArgs) {
        logicgates::display::render(&mut self.gl, render_args, &self.circuit, &self.locations, &self.current_input)
    }

    fn update(&mut self, update_args: &piston::UpdateArgs) {
        if self.wait > 1.0 {
            self.wait = 0.0;
            self.current_input = self.inputs.next().unwrap();
        }
        self.wait += update_args.dt;

        self.locations = logicgates::position::evolution::position_iterative(&self.circuit, &self.locations);
    }
}

fn main() {
    let opengl = opengl_graphics::OpenGL::V3_2;

    let mut window: glutin_window::GlutinWindow = piston::WindowSettings::new("logic gates", [1280, 720]).graphics_api(opengl).resizable(true).samples(4).exit_on_esc(true).build().unwrap();

    let mut app = App {
        gl: opengl_graphics::GlGraphics::new(opengl),
        circuit: logicgates::circuit::Circuit {
            arity: 3,
            gates: vec![
                logicgates::circuit::Gate::And(logicgates::circuit::Value::Arg(0), logicgates::circuit::Value::Arg(1)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::GateValue(0, 0), logicgates::circuit::Value::Arg(2)),
                logicgates::circuit::Gate::And(logicgates::circuit::Value::Arg(0), logicgates::circuit::Value::Arg(1)),
            ],
            output: vec![logicgates::circuit::Value::GateValue(1, 0)],
        },
        locations: vec![[60.0, 60.0], [100.0, 50.0], [500.0, 500.0]],
        inputs: logicgates::eval::enumerate_inputs(3).into_iter().cycle(),
        current_input: vec![false, false, true],
        wait: 0.0,
    };

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
