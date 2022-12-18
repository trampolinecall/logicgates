use piston::{self, PressEvent, RenderEvent, UpdateEvent};

pub struct App {
    gl: opengl_graphics::GlGraphics,
    simulation: logicgates::simulation::Simulation,
    input: Vec<bool>,
}

impl App {
    fn new(gl: opengl_graphics::GlGraphics, simulation: logicgates::simulation::Simulation) -> App {
        let input = (0..simulation.circuit.num_inputs).map(|_| false).collect();
        App { gl, simulation, input }
    }

    fn render(&mut self, render_args: &piston::RenderArgs) {
        self.simulation.render(&mut self.gl, render_args, &self.input);
    }

    fn update(&mut self, _: &piston::UpdateArgs) {}
}

fn main() {
    let circuit = logicgates::compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap();
    let opengl = opengl_graphics::OpenGL::V3_2;

    let mut window: glutin_window::GlutinWindow = piston::WindowSettings::new("logic gates", [1280, 720]).graphics_api(opengl).resizable(true).samples(4).exit_on_esc(true).build().unwrap();

    let mut app = App::new(opengl_graphics::GlGraphics::new(opengl), logicgates::simulation::Simulation::new(circuit));

    let mut events = piston::Events::new(piston::EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(piston::Button::Keyboard(key)) = e.press_args() {
            if let Some(index) = match key {
                piston::Key::D1 => Some(0),
                piston::Key::D2 => Some(1),
                piston::Key::D3 => Some(2),
                piston::Key::D4 => Some(3),
                piston::Key::D5 => Some(4),
                piston::Key::D6 => Some(5),
                piston::Key::D7 => Some(6),
                piston::Key::D8 => Some(7),
                piston::Key::D9 => Some(8),
                piston::Key::D0 => Some(9),
                piston::Key::A => Some(10),
                piston::Key::B => Some(11),
                piston::Key::C => Some(12),
                piston::Key::D => Some(13),
                piston::Key::E => Some(14),
                piston::Key::F => Some(15),
                piston::Key::G => Some(16),
                piston::Key::H => Some(17),
                piston::Key::I => Some(18),
                piston::Key::J => Some(19),
                piston::Key::K => Some(20),
                piston::Key::L => Some(21),
                piston::Key::M => Some(22),
                piston::Key::N => Some(23),
                piston::Key::O => Some(24),
                piston::Key::P => Some(25),
                piston::Key::Q => Some(26),
                piston::Key::R => Some(27),
                piston::Key::S => Some(28),
                piston::Key::T => Some(29),
                piston::Key::U => Some(30),
                piston::Key::V => Some(31),
                piston::Key::W => Some(32),
                piston::Key::X => Some(33),
                piston::Key::Y => Some(34),
                piston::Key::Z => Some(35),

                _ => None,
            } {
                if index < app.input.len() {
                    app.input[index] = !app.input[index];
                }
            }
        }
    }
}
