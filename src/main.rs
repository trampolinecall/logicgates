#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub(crate) mod utils;
pub(crate) mod compiler;
pub(crate) mod simulation;
pub(crate) mod ui;

use nannou::prelude::*;

fn main() {
    nannou::app(model).event(event).update(update).simple_window(view).run();
}

fn model(_: &App) -> simulation::Simulation {
    compiler::compile(&std::env::args().nth(1).expect("expected input file")).unwrap()
}

fn event(_: &App, _: &mut simulation::Simulation, _: Event) {}

fn update(_: &App, simulation: &mut simulation::Simulation, _: Update) {
    simulation::logic::update(&mut simulation.gates, &mut simulation.nodes);
}

fn view(app: &App, simulation: &simulation::Simulation, frame: Frame) {
    let draw = app.draw();
    ui::render(app, &draw, simulation);
    draw.to_frame(app, &frame).unwrap();
}
