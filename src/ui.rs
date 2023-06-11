use crate::simulation::Simulation;

pub(crate) mod gate;
pub(crate) mod node;
pub(crate) mod simulation;

// mvc pattern inspired by elm architecture
pub(crate) struct View {
    sim: simulation::SimulationWidget
}

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect);
}

pub(crate) fn view(simulation: &Simulation) -> View {
    todo!()
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, view: &View) {
    todo!()
    // view.sim.draw(view, draw, app.window_rect());
}
