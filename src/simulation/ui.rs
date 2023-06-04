use crate::simulation::Simulation;

pub(crate) mod simulation;

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect);
}

pub(crate) struct GateWidget {}
pub(crate) struct NodeWidget {}

impl GateWidget {
    pub(crate) fn new() -> GateWidget {
        GateWidget {}
    }
}
impl NodeWidget {
    pub(crate) fn new() -> NodeWidget {
        NodeWidget {}
    }
}

impl Widget for GateWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect) {
        todo!()
    }
}
impl Widget for NodeWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect) {
        todo!()
    }
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation) {
    simulation.widget.draw(simulation, draw, app.window_rect())
}
