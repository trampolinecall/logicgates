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

fn model(_: &App) -> LogicGates {
    let simulation = compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap();
    let view = ui::view(&simulation);
    LogicGates { simulation, view }
}

fn event(_: &App, simulation: &mut LogicGates, event: Event) {
    // logicgates.update_view(); // TODO: add this when adding event handling
}

fn update(_: &App, logicgates: &mut LogicGates, _: Update) {
    simulation::logic::update(&mut logicgates.simulation.gates, &mut logicgates.simulation.nodes);
    logicgates.update_view()
}

fn view(app: &App, logicgates: &LogicGates, frame: Frame) {
    let draw = app.draw();
    ui::render(app, &draw, &logicgates.simulation, &logicgates.view);
    draw.to_frame(app, &frame).unwrap();
}
