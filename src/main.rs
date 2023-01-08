#![allow(clippy::upper_case_acronyms)]
// #![allow(unreachable_code, dead_code, unused_variables, unused_imports, unused_mut)] // TODO: remove this when overhaul is done

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;

pub(crate) struct App {
    gl: opengl_graphics::GlGraphics,
    simulation: simulation::Simulation,
}

impl App {
    fn new(gl: opengl_graphics::GlGraphics, simulation: simulation::Simulation) -> App {
        App { gl, simulation }
    }

    fn render(&mut self, render_args: &piston::RenderArgs) {
        self.simulation.render(&mut self.gl, render_args); // TODO: make the render component manager store gl?
    }

    fn update(&mut self, _: piston::UpdateArgs) {
        self.simulation.update();
    }
}

fn main() {
    // let circuit = compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap(); TODO
    let opengl = opengl_graphics::OpenGL::V3_2;

    let mut window: glutin_window::GlutinWindow = piston::WindowSettings::new("logic gates", [1280, 720]).graphics_api(opengl).resizable(true).samples(4).exit_on_esc(true).build().unwrap();

    let mut app = App::new(opengl_graphics::GlGraphics::new(opengl), {
        let mut gates = generational_arena::Arena::new();

        let nand = gates.insert_with(|index| simulation::Gate {
            index,
            calculation: simulation::components::calculator::CalculationComponent::new_nand(index),
            draw_component: simulation::components::draw::DrawComponent { position: (0, 0.0) },
        });

        let circuit = gates.insert_with(|index| simulation::Gate {
            index,
            calculation: simulation::components::calculator::CalculationComponent::new_custom(
                index,
                simulation::Circuit {
                    name: "main".into(),
                    gates: vec![nand],
                    inputs: simulation::components::connection::ProducersComponent(vec![
                        simulation::components::connection::Producer::new(false),
                        simulation::components::connection::Producer::new(true),
                    ]),
                    outputs: simulation::components::connection::ReceiversComponent(vec![simulation::components::connection::Receiver::new()]),
                },
            ),
            draw_component: simulation::components::draw::DrawComponent { position: (0, 0.0) },
        });

        simulation::Simulation { gates, main_circuit: circuit }
    });

    let mut events = piston::Events::new(piston::EventSettings { ups: 20, ..Default::default() });
    while let Some(e) = events.next(&mut window) {
        use piston::{PressEvent, RenderEvent, UpdateEvent};
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        /* TODO: probably move this to userinput component
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
                if index < app.circuit.num_inputs() {
                    app.circuit.toggle_input(index);
                }
            }
        }
        */

        if let Some(args) = e.update_args() {
            app.update(args);
        }
    }
}
