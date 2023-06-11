#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod ui;

use nannou::prelude::*;

struct LogicGates {
    simulation: simulation::Simulation, // model in the mvc pattern terminology
    view: ui::View,
}
impl LogicGates {
    fn update_view(&mut self) {
        self.view = ui::view(&self.simulation);
    }
}

fn main() {
    nannou::app(model).event(event).update(update).simple_window(view).run();
}

fn event(_: &App, simulation: &mut LogicGates, event: Event) {
    /*
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
            if index < simulation.circuits[simulation.main_circuit].nodes.inputs().len() {
                simulation::logic::toggle_input(&mut simulation.circuits, &mut simulation.nodes, simulation.main_circuit, index);
            }
        }
    }
    */
    // TODO: remove?
    // logicgates.update_view(); // TODO: add this when adding event handling
}

fn model(_: &App) -> LogicGates {
    let simulation = compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap();
    let view = ui::view(&simulation);
    LogicGates { simulation, view }
}

fn update(_: &App, logicgates: &mut LogicGates, _: Update) {
    simulation::logic::update(&mut logicgates.simulation.gates, &mut logicgates.simulation.nodes);
    logicgates.update_view()
}

fn view(app: &App, logicgates: &LogicGates, frame: Frame) {
    let draw = app.draw();
    ui::render(app, &draw, &logicgates.view);
    draw.to_frame(app, &frame).unwrap();
}
