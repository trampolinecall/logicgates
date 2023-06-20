#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod newui;
pub(crate) mod newview;
pub(crate) mod simulation;
pub(crate) mod theme;
pub(crate) mod ui;

use nannou::prelude::*;

// TODO: find a better place to put this and reorganize everything
struct LogicGates {
    simulation: simulation::Simulation,
    subticks_per_update: isize,
    newui: newui::UI,
}

impl LogicGates {
    fn new(_: &App) -> LogicGates {
        LogicGates { simulation: compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap(), subticks_per_update: 1, newui: newui::UI::new() }
    }

}

fn main() {
    nannou::app(LogicGates::new).event(event).update(update).simple_window(view).run();
}

fn event(app: &App, logic_gates: &mut LogicGates, event: Event) {
    newview::event(app, logic_gates, event);
}

fn update(_: &App, logic_gates: &mut LogicGates, _: Update) {
    // TODO: adjust number of ticks for time since last update
    simulation::logic::update(&mut logic_gates.simulation.gates, &mut logic_gates.simulation.nodes, logic_gates.subticks_per_update as usize);
}

fn view(app: &App, logic_gates: &LogicGates, frame: Frame) {
    let draw = app.draw();
    newview::render(app, &draw, logic_gates);
    draw.to_frame(app, &frame).unwrap();
}
