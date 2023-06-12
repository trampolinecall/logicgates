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

fn main() {
    nannou::app(model).event(event).update(update).simple_window(view).run();
}

fn model(_: &App) -> simulation::Simulation {
    compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap()
}

fn event(app: &App, simulation: &mut simulation::Simulation, event: Event) {
    let message = view::event(app, simulation, event);
    if let Some(message) = message {
        simulation.message(message);
    }
}

fn update(_: &App, simulation: &mut simulation::Simulation, _: Update) {
    // TODO: adjust number of ticks for time since last update
    simulation::logic::update(&mut simulation.gates, &mut simulation.nodes);
}

fn view(app: &App, simulation: &simulation::Simulation, frame: Frame) {
    let draw = app.draw();
    view::render(app, &draw, simulation);
    draw.to_frame(app, &frame).unwrap();
}
