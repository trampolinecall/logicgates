#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod view;

use nannou::prelude::*;

// TODO: find a better place to put this and reorganize everything (possibly make LogicGates struct instead of using simulation as model)
enum Message {
    GateDragged(simulation::GateKey, Vec2),
}

struct LogicGates {
    simulation: simulation::Simulation,
    // ui: ui::UI, TODO
}

impl LogicGates {
    fn new(_: &App) -> LogicGates {
        LogicGates { simulation: compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap() }
    }

    fn message(&mut self, message: crate::Message) {
        match message {
            Message::GateDragged(gate, mouse_loc) => {
                let loc = simulation::Gate::location_mut(&mut self.simulation.circuits, &mut self.simulation.gates, gate);
                loc.x = mouse_loc.x; // TODO: zooming
                loc.y = mouse_loc.y;
            }
        }
    }
}

fn main() {
    nannou::app(LogicGates::new).event(event).update(update).simple_window(view).run();
}

fn event(app: &App, logic_gates: &mut LogicGates, event: Event) {
    let message = view::event(app, &logic_gates.simulation, event);
    if let Some(message) = message {
        logic_gates.message(message);
    }
}

fn update(_: &App, logic_gates: &mut LogicGates, _: Update) {
    // TODO: adjust number of ticks for time since last update
    simulation::logic::update(&mut logic_gates.simulation.gates, &mut logic_gates.simulation.nodes);
}

fn view(app: &App, logic_gates: &LogicGates, frame: Frame) {
    let draw = app.draw();
    view::render(app, &draw, &logic_gates.simulation);
    draw.to_frame(app, &frame).unwrap();
}
