use crate::simulation::Simulation;

pub(crate) mod gate;
pub(crate) mod node;
pub(crate) mod simulation;

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect);
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation) {
    simulation.widget.draw(simulation, draw, app.window_rect())
}
