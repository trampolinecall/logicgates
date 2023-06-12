use crate::{simulation::Simulation, view::simulation::SimulationWidget};

mod connection;
mod gate;
mod node;
mod simulation;

// mvc pattern inspired by elm architecture
pub(crate) struct View {
    sim: simulation::SimulationWidget,
}

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, hovered: Option<&dyn Widget>);
    // iterate through this and child widgets in z order to check which one the mouse is currently over
    fn find_hover(&self, mouse: nannou::geom::Vec2) -> Option<&dyn Widget>;
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation) {
    let view = view(app, simulation);
    let hover = view.sim.find_hover(app.mouse.position());
    view.sim.draw(simulation, draw, hover);
}

fn view(app: &nannou::App, simulation: &Simulation) -> View {
    View { sim: SimulationWidget::new(app.window_rect(), simulation) }
}
