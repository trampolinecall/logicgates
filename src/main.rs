#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).update(update).simple_window(view).run();
}

fn event(_: &App, simulation: &mut simulation::Simulation, event: Event) {
    if let Event::WindowEvent { simple: Some(KeyPressed(key)), .. } = event {
        if let Some(index) = match key {
            Key::Key1 => Some(0),
            Key::Key2 => Some(1),
            Key::Key3 => Some(2),
            Key::Key4 => Some(3),
            Key::Key5 => Some(4),
            Key::Key6 => Some(5),
            Key::Key7 => Some(6),
            Key::Key8 => Some(7),
            Key::Key9 => Some(8),
            Key::Key0 => Some(9),
            Key::A => Some(10),
            Key::B => Some(11),
            Key::C => Some(12),
            Key::D => Some(13),
            Key::E => Some(14),
            Key::F => Some(15),
            Key::G => Some(16),
            Key::H => Some(17),
            Key::I => Some(18),
            Key::J => Some(19),
            Key::K => Some(20),
            Key::L => Some(21),
            Key::M => Some(22),
            Key::N => Some(23),
            Key::O => Some(24),
            Key::P => Some(25),
            Key::Q => Some(26),
            Key::R => Some(27),
            Key::S => Some(28),
            Key::T => Some(29),
            Key::U => Some(30),
            Key::V => Some(31),
            Key::W => Some(32),
            Key::X => Some(33),
            Key::Y => Some(34),
            Key::Z => Some(35),

            _ => None,
        } {
            if index < simulation.circuits[simulation.main_circuit].num_inputs() {
                simulation::logic::toggle_input(&mut simulation.circuits, &mut simulation.nodes, simulation.main_circuit, index);
            }
        }
    }
}

fn model(_: &App) -> simulation::Simulation {
    compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap()
}

fn update(_: &App, simulation: &mut simulation::Simulation, _: Update) {
    simulation::logic::update(&mut simulation.gates, &mut simulation.nodes);
}

fn view(app: &App, simulation: &simulation::Simulation, frame: Frame) {
    let draw = app.draw();
    simulation::draw::render(app, &draw, simulation, simulation.main_circuit);
    draw.to_frame(app, &frame).unwrap();
}
