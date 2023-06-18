#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod ui;
pub(crate) mod view;

use nannou::prelude::*;

// TODO: find a better place to put this and reorganize everything
struct LogicGates {
    simulation: simulation::Simulation,
    ui: ui::UI,
}

// TODO: find a better place to put this too
enum Message {
    MouseDownOnGate(simulation::GateKey),
    MouseMoved(Vec2),
    MouseUp,
}

impl LogicGates {
    fn new(_: &App) -> LogicGates {
        LogicGates { simulation: compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap(), ui: ui::UI::new() }
    }

    fn message(&mut self, message: crate::Message) {
        /* TODO
        match message {
            Message::MouseDownOnGate(gate) => {
                self.ui.main_widget.cur_gate_drag = Some(gate);
            }
            Message::MouseMoved(mouse_pos) => {
                if let Some(cur_gate_drag) = self.ui.main_widget.cur_gate_drag {
                    let loc = simulation::Gate::location_mut(&mut self.simulation.circuits, &mut self.simulation.gates, cur_gate_drag);
                    loc.x = mouse_pos.x; // TODO: zooming
                    loc.y = mouse_pos.y;
                }
            }
            Message::MouseUp => {
                self.ui.main_widget.cur_gate_drag = None;
            }
        }
        */
    }
}

fn main() {
    nannou::app(LogicGates::new).event(event).update(update).simple_window(view).run();
}

fn event(app: &App, logic_gates: &mut LogicGates, event: Event) {
    let message = view::event(app, logic_gates, event);
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
    view::render(app, &draw, logic_gates);
    draw.to_frame(app, &frame).unwrap();
}
