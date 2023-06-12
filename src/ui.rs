use crate::{simulation::Simulation, ui::simulation::SimulationWidget};

mod connection;
mod gate;
mod node;
mod simulation;

// mvc pattern inspired by elm architecture
pub(crate) struct View {
    sim: simulation::SimulationWidget,
}

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw);
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation) {
    view(app, simulation).sim.draw(simulation, draw);
}

fn view(app: &nannou::App, simulation: &Simulation) -> View {
    View { sim: SimulationWidget::new(app.window_rect(), simulation) }
}
