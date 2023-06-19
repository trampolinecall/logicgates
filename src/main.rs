#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod ui;
pub(crate) mod view;
pub(crate) mod theme;

use crate::ui::widgets::Widget;

use nannou::prelude::*;

// TODO: find a better place to put this and reorganize everything
struct LogicGates {
    simulation: simulation::Simulation,
    ui: ui::UI,
}

// TODO: find a better place to put this too
enum Message {
    GateMoved(simulation::GateKey, Vec2),
}

impl LogicGates {
    fn new(_: &App) -> LogicGates {
        LogicGates { simulation: compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap(), ui: ui::UI::new() }
    }

    fn message(&mut self, message: crate::Message) {
        match message {
            Message::GateMoved(gate, pos) => {
                let loc = simulation::Gate::location_mut(&mut self.simulation.circuits, &mut self.simulation.gates, gate);
                loc.x = pos.x;
                loc.y = pos.y;
            }
        }
    }

    fn view(&self, rect: nannou::geom::Rect) -> (Box<dyn view::Drawing>, Vec<view::Subscription>) {
        self.ui.main_widget.view(self, rect)
    }
}

fn main() {
    nannou::app(LogicGates::new).event(event).update(update).simple_window(view).run();
}

fn event(app: &App, logic_gates: &mut LogicGates, event: Event) {
    let ui_message = view::event(app, logic_gates, event);
    for ui_message in ui_message {
        let logic_gate_message = logic_gates.ui.targeted_message(ui_message);
        if let Some(logic_gate_message) = logic_gate_message {
            logic_gates.message(logic_gate_message);
        }
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
